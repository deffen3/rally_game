use amethyst::{
    core::{Transform, Time},
    derive::SystemDesc,
    ecs::{World, Join, Read, System, SystemData, WriteStorage, ReadStorage, ReadExpect, Entities},
    assets::AssetStorage,
    audio::{output::Output, Source},
};

use crate::rally::{WeaponFire, Vehicle, Player, vehicle_damage_model};

use std::ops::Deref;
use crate::audio::{play_score_sound, Sounds};


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
        ReadStorage<'s, WeaponFire>,
        Read<'s, Time>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn setup(&mut self, _world: &mut World) {
        self.hit_sound_cooldown_timer = -1.0;
    }

    fn run(&mut self, (entities, transforms, players, mut vehicles, weapon_fires,
            time, storage, sounds, audio_output): Self::SystemData) {
        let dt = time.delta_seconds();

        for (vehicle_entity, player, vehicle, vehicle_transform) in (&*entities, &players, &mut vehicles, &transforms).join() {
            let vehicle_x = vehicle_transform.translation().x;
            let vehicle_y = vehicle_transform.translation().y;

            for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in (&*entities, &weapon_fires, &transforms).join() {
                let fire_x = weapon_fire_transform.translation().x;
                let fire_y = weapon_fire_transform.translation().y;

                if weapon_fire.owner_player_id != player.id {
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
                        }

                        if self.hit_sound_cooldown_timer < 0.0 {
                            play_score_sound(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
                            self.hit_sound_cooldown_timer = HIT_SOUND_COOLDOWN_RESET;
                        }
                    }
                }
            }
        }

        self.hit_sound_cooldown_timer -= dt;
    }
}