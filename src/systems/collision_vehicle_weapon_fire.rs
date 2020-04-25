use amethyst::{
    core::{Transform, Time},
    derive::SystemDesc,
    ecs::{World, Join, Read, System, SystemData, WriteStorage, ReadStorage, ReadExpect, Entities},
    assets::AssetStorage,
    audio::{output::Output, Source},
};

use crate::components::{
    WeaponFire, Weapon, WeaponTypes, Vehicle, Player, 
    Hitbox,
    kill_restart_vehicle,
    get_next_weapon_type, update_weapon_properties,
};

use crate::rally::{vehicle_damage_model};


use std::ops::Deref;
use crate::audio::{play_score_sound, play_bounce_sound, Sounds};


pub const HIT_SOUND_COOLDOWN_RESET: f32 = 0.25;

#[derive(SystemDesc, Default)]
pub struct CollisionVehicleWeaponFireSystem {
    pub hit_sound_cooldown_timer: f32,
}

impl<'s> System<'s> for CollisionVehicleWeaponFireSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Hitbox>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Weapon>,
        ReadStorage<'s, WeaponFire>,
        Read<'s, Time>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn setup(&mut self, _world: &mut World) {
        self.hit_sound_cooldown_timer = -1.0;
    }

    fn run(&mut self, (entities, hitboxes, mut transforms, mut players, mut vehicles, mut weapons, weapon_fires,
            time, storage, sounds, audio_output): Self::SystemData) {
        let dt = time.delta_seconds();


        for (hitbox, transform) in (&hitboxes, &transforms).join() {
            let hitbox_x = transform.translation().x;
            let hitbox_y = transform.translation().y;

            for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in (&*entities, &weapon_fires, &transforms).join() {
                let fire_x = weapon_fire_transform.translation().x;
                let fire_y = weapon_fire_transform.translation().y;

                if (fire_x - hitbox_x).powi(2) + (fire_y - hitbox_y).powi(2) < (hitbox.width/2.0).powi(2) {
                    if weapon_fire.attached == false {
                        let _ = entities.delete(weapon_fire_entity);
                    }
                }
            }
        }


        let mut player_makes_kill: Vec<(usize, usize, WeaponTypes)> = Vec::new();

        for (player, vehicle, _weapon, vehicle_transform) in (&players, &mut vehicles, &mut weapons, &transforms).join() {
            let vehicle_x = vehicle_transform.translation().x;
            let vehicle_y = vehicle_transform.translation().y;

            for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in (&*entities, &weapon_fires, &transforms).join() {
                let fire_x = weapon_fire_transform.translation().x;
                let fire_y = weapon_fire_transform.translation().y;

                if weapon_fire.owner_player_id != player.id {

                    let fire_rotation = weapon_fire_transform.rotation();
                    let (_, _, fire_angle) = fire_rotation.euler_angles();
                    let fire_x_comp = -fire_angle.sin(); //left is -, right is +
                    let fire_y_comp = fire_angle.cos(); //up is +, down is -

                    // let vehicle_rotation = vehicle_transform.rotation();
                    // let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();
                    // let vehicle_x_comp = -vehicle_angle.sin(); //left is -, right is +
                    // let vehicle_y_comp = vehicle_angle.cos(); //up is +, down is -


                    if ((fire_x - vehicle_x).powi(2) + (fire_y - vehicle_y).powi(2) < vehicle.width.powi(2)) || 
                        ((fire_x + fire_x_comp*weapon_fire.height/2.0 - vehicle_x).powi(2) + 
                            (fire_y + fire_y_comp*weapon_fire.height/2.0 - vehicle_y).powi(2) < vehicle.width.powi(2)) ||
                        ((fire_x - fire_x_comp*weapon_fire.height/2.0 - vehicle_x).powi(2) + 
                            (fire_y - fire_y_comp*weapon_fire.height/2.0 - vehicle_y).powi(2) < vehicle.width.powi(2))
                    {

                        if weapon_fire.attached == false {
                            let _ = entities.delete(weapon_fire_entity);
                        }

                        let damage:f32 = weapon_fire.damage.clone();

                        let vehicle_destroyed:bool = vehicle_damage_model(vehicle, damage, 
                            weapon_fire.piercing_damage_pct, 
                            weapon_fire.shield_damage_pct,
                            weapon_fire.armor_damage_pct,
                            weapon_fire.health_damage_pct
                        );

                        if vehicle_destroyed {
                            play_bounce_sound(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));

                            player_makes_kill.push((weapon_fire.owner_player_id.clone(), player.id.clone(), weapon_fire.weapon_type.clone()));
                        }

                        if self.hit_sound_cooldown_timer < 0.0 {
                            play_score_sound(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
                            self.hit_sound_cooldown_timer = HIT_SOUND_COOLDOWN_RESET;
                        }
                    }
                }
            }
        }

        for (entity, player, weapon, vehicle, transform) in (&*entities, &mut players, &mut weapons, &mut vehicles, &mut transforms).join() {

            for (killer_id, killed_id, weapon_type) in &player_makes_kill {
                if *killer_id == player.id {
                    //classic gun-game rules: upgrade weapon type for player who got the kill
                    let new_weapon_type = get_next_weapon_type(weapon_type.clone());

                    player.kills += 1;

                    if let Some(some_weapon_type) = new_weapon_type {
                        println!("{:?} {:?}",weapon_type.clone(), some_weapon_type);
                        update_weapon_properties(weapon, some_weapon_type);
                    }
                }

                if *killed_id == player.id {
                    kill_restart_vehicle(vehicle, transform);
                }
                
            }
        }

        self.hit_sound_cooldown_timer -= dt;
    }
}
