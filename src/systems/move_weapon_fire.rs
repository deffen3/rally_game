use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage, ReadStorage, Entities};

use crate::components::{WeaponFire, Vehicle, Player};
use crate::rally::{ARENA_WIDTH, ARENA_HEIGHT, UI_HEIGHT};

use std::f32::consts::PI;


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

        let mut vehicle_owner_x = 40.0;
        let mut vehicle_owner_y = 40.0;
        let mut vehicle_owner_angle = 0.0;
        let mut heat_seeking_angle = 0.0;

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

                        let weapon_rotation = transform.rotation();
                        let (_, _, weapon_angle) = weapon_rotation.euler_angles();

                        let dist = ((vehicle_x - fire_x).powi(2) + (vehicle_y - fire_y).powi(2)).sqrt();

                        if dist < closest_vehicle_dist {
                            closest_vehicle_dist = dist.clone();
                            closest_vehicle_x_diff = fire_x - vehicle_x;
                            closest_vehicle_y_diff = fire_y - vehicle_y;
                        }
                    }
                }

                let target_angle = closest_vehicle_y_diff.atan2(closest_vehicle_x_diff) + (PI/2.0); //rotate by PI/2 to line up with yaw angle
                let velocity_angle = weapon_fire.dy.atan2(weapon_fire.dx) + (PI/2.0);
                heat_seeking_angle = target_angle;
            }
            
            if weapon_fire.attached {
                for (vehicle, vehicle_transform, player) in (&vehicles, &transforms, &players).join() {
                    if weapon_fire.owner_player_id == player.id {
                        vehicle_owner_x = vehicle_transform.translation().x;
                        vehicle_owner_y = vehicle_transform.translation().y;

                        let vehicle_rotation = vehicle_transform.rotation();
                        let (_, _, yaw) = vehicle_rotation.euler_angles();

                        vehicle_owner_angle = yaw;
                    }
                }
            }

            
        }


        for (entity, weapon_fire, transform) in (&*entities, &mut weapon_fires, &mut transforms).join() {
            if weapon_fire.heat_seeking == true {
                transform.set_rotation_2d(heat_seeking_angle);

                let velocity_x_comp = -heat_seeking_angle.sin(); //left is -, right is +
                let velocity_y_comp = heat_seeking_angle.cos(); //up is +, down is -

                let sq_vel = weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2);
                let abs_vel = sq_vel.sqrt();

                weapon_fire.dx += (weapon_fire.heat_seeking_agility * velocity_x_comp * dt);
                weapon_fire.dx *= weapon_fire.weapon_shot_speed / abs_vel;

                weapon_fire.dy += (weapon_fire.heat_seeking_agility * velocity_y_comp * dt);
                weapon_fire.dy *= weapon_fire.weapon_shot_speed / abs_vel;

                let sq_vel2 = weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2);
                let abs_vel2 = sq_vel2.sqrt();

                //println!("{} : {}", abs_vel, abs_vel2);
            }

            if weapon_fire.attached == true {
                if weapon_fire.deployed == true {
                    let yaw_x_comp = -vehicle_owner_angle.sin(); //left is -, right is +
                    let yaw_y_comp = vehicle_owner_angle.cos(); //up is +, down is -
    
                    //println!("attached: {}, {}, {}",vehicle_owner_x, vehicle_owner_y, vehicle_owner_angle);
    
                    transform.set_rotation_2d(vehicle_owner_angle - PI);
                    transform.set_translation_x(vehicle_owner_x - yaw_x_comp*14.0);
                    transform.set_translation_y(vehicle_owner_y - yaw_y_comp*14.0);
                }
            }
            else {
                transform.prepend_translation_x(weapon_fire.dx * dt);
                transform.prepend_translation_y(weapon_fire.dy * dt);

                let fire_x = transform.translation().x;
                let fire_y = transform.translation().y;

                //out of arena logic
                if (fire_x > (ARENA_WIDTH + 2.0*weapon_fire.width)) || 
                        (fire_x < (-2.0*weapon_fire.width)) || 
                        (fire_y > (ARENA_HEIGHT + 2.0*weapon_fire.width)) ||
                        (fire_y < (UI_HEIGHT - -2.0*weapon_fire.width)) {

                    if weapon_fire.attached == false {
                        let _ = entities.delete(entity);
                    }
                }
            }
        }
    }
}