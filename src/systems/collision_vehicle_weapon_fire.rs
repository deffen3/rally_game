use amethyst::{
    core::{Transform, Time},
    derive::SystemDesc,
    ecs::{World, Join, Read, System, SystemData, WriteStorage, ReadStorage, ReadExpect, Entities},
    assets::AssetStorage,
    audio::{output::Output, Source},
};

use crate::components::{
    WeaponFire, Weapon, WeaponTypes, Vehicle, Player, 
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
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Player>,
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

    fn run(&mut self, (entities, transforms, players, mut vehicles, mut weapons, weapon_fires,
            time, storage, sounds, audio_output): Self::SystemData) {
        let dt = time.delta_seconds();

        let mut player_makes_kill: Vec<(usize, WeaponTypes)> = Vec::new();

        for (vehicle_entity, player, vehicle, weapon, vehicle_transform) in (&*entities, &players, &mut vehicles, &mut weapons, &transforms).join() {
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

                    let vehicle_rotation = vehicle_transform.rotation();
                    let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();
                    let vehicle_x_comp = -vehicle_angle.sin(); //left is -, right is +
                    let vehicle_y_comp = vehicle_angle.cos(); //up is +, down is -

                    

                    if (fire_x - vehicle_x).powi(2) + (fire_y - vehicle_y).powi(2) < vehicle.width.powi(2) {

                        let _ = entities.delete(weapon_fire_entity);

                        let damage:f32 = weapon_fire.damage.clone();

                        let vehicle_destroyed:bool = vehicle_damage_model(vehicle, damage, 
                            weapon_fire.piercing_damage_pct, 
                            weapon_fire.shield_damage_pct,
                            weapon_fire.armor_damage_pct,
                            weapon_fire.health_damage_pct
                        );

                        if vehicle_destroyed {
                            let _ = entities.delete(vehicle_entity);
                            play_bounce_sound(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));

                            player_makes_kill.push((weapon_fire.owner_player_id.clone(), weapon_fire.weapon_type.clone()));
                        }

                        if self.hit_sound_cooldown_timer < 0.0 {
                            play_score_sound(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
                            self.hit_sound_cooldown_timer = HIT_SOUND_COOLDOWN_RESET;
                        }
                    }
                }
            }
        }

        for (player, weapon) in (&players, &mut weapons).join() {

            for (killer_id, weapon_type) in &player_makes_kill {
                if *killer_id == player.id {
                    //classic gun-game rules: upgrade weapon type for player who got the kill
                    let new_weapon_type = get_next_weapon_type(weapon_type.clone());
                    println!("{:?} {:?}",weapon_type.clone(), new_weapon_type);
                    update_weapon_properties(weapon, new_weapon_type);
                }
            }
        }

        self.hit_sound_cooldown_timer -= dt;
    }
}
