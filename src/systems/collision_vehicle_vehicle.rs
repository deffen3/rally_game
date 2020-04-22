use amethyst::{
    core::{Transform, Time},
    derive::SystemDesc,
    ecs::{Join, Read, System, SystemData, WriteStorage, ReadStorage, ReadExpect, Entities},
    assets::AssetStorage,
    audio::{output::Output, Source},
};

use std::f32::consts::PI;

use crate::rally::{Vehicle, Player, vehicle_damage_model, BASE_COLLISION_DAMAGE, 
    COLLISION_PIERCING_DAMAGE_PCT, COLLISION_SHIELD_DAMAGE_PCT,
    COLLISION_ARMOR_DAMAGE_PCT, COLLISION_HEALTH_DAMAGE_PCT};

use std::ops::Deref;
use crate::audio::{play_bounce_sound, Sounds};


#[derive(SystemDesc, Default)]
pub struct CollisionVehToVehSystem;


impl<'s> System<'s> for CollisionVehToVehSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        Read<'s, Time>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(&mut self, (entities, mut transforms, players, mut vehicles,
            time, storage, sounds, audio_output): Self::SystemData) {
        let dt = time.delta_seconds();

        let mut collision_ids_vec: Vec<(usize, f32)> = Vec::new();

        for (vehicle_1, player_1, vehicle_1_transform) in (&vehicles, &players, &transforms).join() {
            let vehicle_1_x = vehicle_1_transform.translation().x;
            let vehicle_1_y = vehicle_1_transform.translation().y;

            for (vehicle_2, player_2, vehicle_2_transform) in (&vehicles, &players, &transforms).join() {
                let vehicle_2_x = vehicle_2_transform.translation().x;
                let vehicle_2_y = vehicle_2_transform.translation().y;

                if player_1.id != player_2.id {
                    if (vehicle_1_x - vehicle_2_x).powi(2) + (vehicle_1_y - vehicle_2_y).powi(2) < vehicle_1.width.powi(2) {

                        let velocity_1_angle = vehicle_1.dy.atan2(vehicle_1.dx) - (PI/2.0); //rotate by PI/2 to line up with yaw angle
                        let velocity_1_x_comp = -velocity_1_angle.sin(); //left is -, right is +
                        let velocity_1_y_comp = velocity_1_angle.cos(); //up is +, down is -

                        //vehicle_1.dx *= VEHICLE_HIT_BOUNCE_DECEL_PCT * velocity_1_x_comp.abs();
                        //vehicle_1.dy *= VEHICLE_HIT_BOUNCE_DECEL_PCT * velocity_1_y_comp.abs();

                        let velocity_2_angle = vehicle_2.dy.atan2(vehicle_2.dx) - (PI/2.0); //rotate by PI/2 to line up with yaw angle
                        let velocity_2_x_comp = -velocity_2_angle.sin(); //left is -, right is +
                        let velocity_2_y_comp = velocity_2_angle.cos(); //up is +, down is -

                        //vehicle_2.dx *= VEHICLE_HIT_BOUNCE_DECEL_PCT * velocity_2_x_comp.abs();
                        //vehicle_2.dy *= VEHICLE_HIT_BOUNCE_DECEL_PCT * velocity_2_y_comp.abs();

                        let sq_vel_diff = (vehicle_1.dx - vehicle_2.dx).powi(2) + (vehicle_1.dy - vehicle_2.dy).powi(2);
                        let abs_vel_diff = sq_vel_diff.sqrt();

                        collision_ids_vec.push((player_1.id, abs_vel_diff));
                        collision_ids_vec.push((player_2.id, abs_vel_diff));
                    }
                }
            }
        }

        for (vehicle_entity, vehicle, player) in (&*entities, &mut vehicles, &players).join() {

            for (col_id, v_diff) in &collision_ids_vec {
                if player.id == *col_id {

                    if vehicle.collision_cooldown_timer <= 0.0 {
                        //println!("Player {} has collided", player.id);

                        play_bounce_sound(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
                        vehicle.collision_cooldown_timer = 1.0;

                        let damage:f32 = BASE_COLLISION_DAMAGE * v_diff;

                        let vehicle_destroyed:bool = vehicle_damage_model(vehicle, damage, 
                            COLLISION_PIERCING_DAMAGE_PCT, COLLISION_SHIELD_DAMAGE_PCT,
                            COLLISION_ARMOR_DAMAGE_PCT, COLLISION_HEALTH_DAMAGE_PCT);

                        if vehicle_destroyed {
                            let _ = entities.delete(vehicle_entity);
                        }
                    }
                    else {
                        vehicle.collision_cooldown_timer -= dt;
                    }
                }
            }
        }

        
    }
}