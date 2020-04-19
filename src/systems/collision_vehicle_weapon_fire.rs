use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, ReadStorage, Entities};

use std::f32::consts::PI;

use crate::rally::{WeaponFire, Vehicle, Player};

#[derive(SystemDesc)]
pub struct CollisionVehicleWeaponFireSystem;

impl<'s> System<'s> for CollisionVehicleWeaponFireSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        ReadStorage<'s, WeaponFire>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, transforms, players, vehicles, weapon_fires, time): Self::SystemData) {
        //let dt = time.delta_seconds();

        for (_vehicle_entity, player, vehicle, vehicle_transform) in (&*entities, &players, &vehicles, &transforms).join() {
            let vehicle_x = vehicle_transform.translation().x;
            let vehicle_y = vehicle_transform.translation().y;

            for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in (&*entities, &weapon_fires, &transforms).join() {
                let fire_x = weapon_fire_transform.translation().x;
                let fire_y = weapon_fire_transform.translation().y;

                if weapon_fire.owner_player_id != player.id {
                    if (fire_x - vehicle_x).powi(2) + (fire_y - vehicle_y).powi(2) < vehicle.width.powi(2) {
                        let _ = entities.delete(weapon_fire_entity);
                    }
                }
            }
        }
    }
}