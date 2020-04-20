use amethyst::{
    core::{Transform, Time},
    derive::SystemDesc,
    ecs::{World, Join, Read, System, SystemData, WriteStorage, ReadStorage, ReadExpect, Entities, Entity, Write},
    assets::AssetStorage,
    audio::{output::Output, Source},
};

use std::f32::consts::PI;

use crate::rally::{WeaponFire, Vehicle, Player};

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

    fn setup(&mut self, world: &mut World) {
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

                        let mut damage:f32 = weapon_fire.damage.clone();
                        let mut piercing_damage:f32 = 0.0;
                        
                        let mut shield_damage_pct:f32 = weapon_fire.shield_damage_pct.clone();
                        let mut armor_damage_pct:f32 = weapon_fire.armor_damage_pct.clone();
                        let mut health_damage_pct:f32 = weapon_fire.health_damage_pct.clone();

                        println!("H:{} A:{} S:{} D:{}",vehicle.health, vehicle.armor, vehicle.shield, damage);

                        let mut piercing_damage_pct:f32 = weapon_fire.piercing_damage_pct.clone();
                        if piercing_damage_pct > 0.0 {
                            piercing_damage = damage * piercing_damage_pct;
                            damage -= piercing_damage;
                        }

                        println!("H:{} A:{} S:{} P:{}, D:{}",vehicle.health, vehicle.armor, vehicle.shield, piercing_damage, damage);

                        if vehicle.shield > 0.0 {
                            vehicle.shield -= (damage * shield_damage_pct);
                            damage = 0.0;

                            if vehicle.shield < 0.0 {
                                damage -= vehicle.shield; //over damage on shields, needs taken from armor
                                vehicle.shield = 0.0;
                            }
                        }

                        println!("H:{} A:{} S:{} D:{}",vehicle.health, vehicle.armor, vehicle.shield, damage);

                        if vehicle.armor > 0.0 {
                            vehicle.armor -= (damage * armor_damage_pct);
                            damage = 0.0;

                            if vehicle.armor < 0.0 {
                                damage -= vehicle.armor; //over damage on armor, needs taken from health
                                vehicle.armor = 0.0;
                            }
                        }

                        println!("H:{} A:{} S:{} D:{}",vehicle.health, vehicle.armor, vehicle.shield, damage);

                        let mut health_damage:f32 = (damage + piercing_damage) * health_damage_pct;

                        if vehicle.health <= health_damage {
                            //vehicle destroyed
                            vehicle.health = 0.0;
                            let _ = entities.delete(vehicle_entity);
                        }
                        else {
                            vehicle.health -= health_damage;
                            health_damage = 0.0;
                        }

                        println!("H:{} A:{} S:{} D:{}",vehicle.health, vehicle.armor, vehicle.shield, health_damage);

    
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

// fn vehicle_damage_model(&mut vehicle, damage, piercing_damage_pct, shield_damage_pct, armor_damage_pct, health_damage_pct) {
//     println!("H:{} A:{} S:{} D:{}",vehicle.health, vehicle.armor, vehicle.shield, damage);
// }