use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage, ReadStorage, Entities};

use crate::components::{WeaponFire, Vehicle, Player};
use crate::rally::{ARENA_WIDTH, ARENA_HEIGHT};


#[derive(SystemDesc)]
pub struct MoveWeaponFireSystem;

impl<'s> System<'s> for MoveWeaponFireSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WeaponFire>,
        ReadStorage<'s, Vehicle>,
        ReadStorage<'s, Player>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, mut transforms, mut weapon_fires, vehicles, players, time): Self::SystemData) {
        let dt = time.delta_seconds();

        for (weapon_fire, transform) in (&mut weapon_fires, &transforms).join() {
            let fire_x = transform.translation().x;
            let fire_y = transform.translation().y;

            if weapon_fire.heat_seeking {
                let mut closest_vehicle_x_diff = 0.0;
                let mut closest_vehicle_y_diff = 0.0;
                let mut closest_vehicle_dist = 1000000000.0;

                for (vehicle, vehicle_transform, player) in (&vehicles, &transforms, &players).join() {
                    if weapon_fire.owner_player_id != player.id {
                        let vehicle_x = vehicle_transform.translation().x;
                        let vehicle_y = vehicle_transform.translation().y;

                        let dist = ((vehicle_x - fire_x).powi(2) + (vehicle_x - fire_x).powi(2)).sqrt();

                        if dist < closest_vehicle_dist {
                            closest_vehicle_dist = dist.clone();
                            closest_vehicle_x_diff = vehicle_x - fire_x;
                            closest_vehicle_y_diff = vehicle_x - fire_x;
                        }
                    }
                }
                
                weapon_fire.dx += closest_vehicle_x_diff * weapon_fire.heat_seeking_agility * dt;
                weapon_fire.dy += closest_vehicle_y_diff * weapon_fire.heat_seeking_agility * dt;
            }
        }

        for (entity, weapon_fire, transform) in (&*entities, &mut weapon_fires, &mut transforms).join() {
            transform.prepend_translation_x(weapon_fire.dx * dt);
            transform.prepend_translation_y(weapon_fire.dy * dt);

            let fire_x = transform.translation().x;
            let fire_y = transform.translation().y;

            //out of arena logic
            if (fire_x > (ARENA_WIDTH + 2.0*weapon_fire.width)) || 
                    (fire_x < (-2.0*weapon_fire.width)) || 
                    (fire_y > (ARENA_HEIGHT + 2.0*weapon_fire.width)) ||
                    (fire_y < (-2.0*weapon_fire.width)) {

                let _ = entities.delete(entity);
            }
        }
    }
}