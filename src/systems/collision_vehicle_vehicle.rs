use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::{Time, Transform},
    derive::SystemDesc,
    ecs::{Join, Read, ReadExpect, System, SystemData, WriteStorage},
};

use crate::audio::{play_bounce_sound, Sounds};
use log::debug;
use std::collections::HashMap;

extern crate nalgebra as na;
use na::{Isometry2, Vector2};
use ncollide2d::query::{self, Proximity};
use ncollide2d::shape::{Cuboid};

use crate::components::{kill_restart_vehicle, vehicle_damage_model, 
    Player, Vehicle, determine_vehicle_weight, DurationDamage,
};

use crate::rally::{
    BASE_COLLISION_DAMAGE, COLLISION_ARMOR_DAMAGE_PCT, COLLISION_HEALTH_DAMAGE_PCT,
    COLLISION_PIERCING_DAMAGE_PCT, COLLISION_SHIELD_DAMAGE_PCT,
};
use crate::resources::{GameModeSetup, GameModes};

const VEHICLE_COLLISION_COOLDOWN_RESET: f32 = 0.1;
const COLLISION_LOSS: f32 = 3.0;

#[derive(SystemDesc, Default)]
pub struct CollisionVehToVehSystem;

impl<'s> System<'s> for CollisionVehToVehSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        Read<'s, Time>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
        ReadExpect<'s, GameModeSetup>,
    );

    fn run(
        &mut self,
        (
            mut transforms,
            mut players,
            mut vehicles,
            time,
            storage,
            sounds,
            audio_output,
            game_mode_setup,
        ): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        let mut collision_ids_map = HashMap::new();

        for (vehicle_1, player_1, vehicle_1_transform) in (&vehicles, &players, &transforms).join()
        {
            let vehicle_1_x = vehicle_1_transform.translation().x;
            let vehicle_1_y = vehicle_1_transform.translation().y;

            let vehicle_1_rotation = vehicle_1_transform.rotation();
            let (_, _, vehicle_1_angle) = vehicle_1_rotation.euler_angles();

            let vehicle_1_collider_shape = Cuboid::new(Vector2::new(vehicle_1.width/2.0, vehicle_1.height/2.0));
            let vehicle_1_collider_pos = Isometry2::new(Vector2::new(vehicle_1_x, vehicle_1_y), vehicle_1_angle);

            let vehicle_1_weight = determine_vehicle_weight(vehicle_1);

            for (vehicle_2, player_2, vehicle_2_transform) in
                (&vehicles, &players, &transforms).join()
            {
                if player_1.id != player_2.id {
                    let vehicle_2_x = vehicle_2_transform.translation().x;
                    let vehicle_2_y = vehicle_2_transform.translation().y;

                    let vehicle_2_rotation = vehicle_2_transform.rotation();
                    let (_, _, vehicle_2_angle) = vehicle_2_rotation.euler_angles();

                    let vehicle_2_collider_shape = Cuboid::new(Vector2::new(vehicle_2.width/2.0, vehicle_2.height/2.0));
                    let vehicle_2_collider_pos = Isometry2::new(Vector2::new(vehicle_2_x, vehicle_2_y), vehicle_2_angle);

                    let collision = query::proximity(
                        &vehicle_1_collider_pos, &vehicle_1_collider_shape,
                        &vehicle_2_collider_pos, &vehicle_2_collider_shape,
                        0.0,
                    );

                    if collision == Proximity::Intersecting {
                        let contact_data = query::contact(
                            &vehicle_1_collider_pos, &vehicle_1_collider_shape,
                            &vehicle_2_collider_pos, &vehicle_2_collider_shape,
                            0.0);

                        let contact_pt = contact_data.unwrap().world2;

                        let vehicle_2_weight = determine_vehicle_weight(vehicle_2);

                        collision_ids_map.insert(player_1.id, (
                            player_2.id,
                            vehicle_2_weight,
                            vehicle_2.dx,
                            vehicle_2.dy,
                            contact_pt,
                        ));
                        
                        collision_ids_map.insert(player_2.id, (
                            player_1.id,
                            vehicle_1_weight,
                            vehicle_1.dx,
                            vehicle_1.dy,
                            contact_pt,
                        ));
                    }
                }
            }
        }

        let mut earned_collision_kills: Vec<usize> = Vec::new();

        for (vehicle, player, transform) in (&mut vehicles, &mut players, &mut transforms).join() {
            let collision_ids = collision_ids_map.get(&player.id);

            if let Some(collision_data) = collision_ids {
                let (other_player_id,
                    other_vehicle_weight,
                    other_vehicle_dx,
                    other_vehicle_dy,
                    contact_pt
                ) = collision_data;

                let sq_vel_diff = (vehicle.dx - other_vehicle_dx).powi(2)
                    + (vehicle.dy - other_vehicle_dy).powi(2);

                let abs_vel_diff = sq_vel_diff.sqrt();

                let vehicle_weight = determine_vehicle_weight(vehicle);

                // let vehicle_momentum = vehicle_weight * 
                //     (vehicle.dx.powi(2) + vehicle.dy.powi(2)).sqrt();

                // let other_vehicle_momentum = other_vehicle_weight * 
                //     (other_vehicle_dx.powi(2) + other_vehicle_dy.powi(2)).sqrt();

                let vehicle_x = transform.translation().x;
                let vehicle_y = transform.translation().y;

                let impulse = COLLISION_LOSS*(2.0 * vehicle_weight)/(vehicle_weight + other_vehicle_weight);

                vehicle.dx = vehicle.dx - impulse*(contact_pt.x - vehicle_x);
                vehicle.dy = vehicle.dy - impulse*(contact_pt.y - vehicle_y);

                transform.set_translation_x(vehicle_x + vehicle.dx*dt);
                transform.set_translation_y(vehicle_y + vehicle.dy*dt);



                if vehicle.collision_cooldown_timer <= 0.0 {
                    debug!("Player {} has collided", player.id);

                    let damage: f32 = BASE_COLLISION_DAMAGE * abs_vel_diff/100.0;

                    if abs_vel_diff > 75.0 {
                        play_bounce_sound(&*sounds, &storage, audio_output.as_deref());
                    }
                    vehicle.collision_cooldown_timer = VEHICLE_COLLISION_COOLDOWN_RESET;

                    let vehicle_destroyed: bool = vehicle_damage_model(
                        vehicle,
                        Some(*other_player_id),
                        None,
                        damage,
                        COLLISION_PIERCING_DAMAGE_PCT,
                        COLLISION_SHIELD_DAMAGE_PCT,
                        COLLISION_ARMOR_DAMAGE_PCT,
                        COLLISION_HEALTH_DAMAGE_PCT,
                        DurationDamage::default(),
                    );

                    if vehicle_destroyed {
                        player.deaths += 1;

                        if player.last_hit_timer <= game_mode_setup.last_hit_threshold {
                            if let Some(last_hit_by_id) = player.last_hit_by_id {
                                earned_collision_kills.push(last_hit_by_id);
                            }
                        }

                        kill_restart_vehicle(
                            player,
                            vehicle,
                            transform,
                            game_mode_setup.stock_lives,
                        );
                    }
                } else {
                    vehicle.collision_cooldown_timer -= dt;
                }
            }
        }

        for player in (&mut players).join() {
            if game_mode_setup.game_mode != GameModes::ClassicGunGame {
                player.kills += earned_collision_kills
                    .iter()
                    .filter(|&n| *n == player.id)
                    .count() as i32;
            }
        }
    }
}
