use amethyst::core::{Time, Transform};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Entities, Join, Read, ReadStorage, System, SystemData, WriteStorage};

use crate::components::{Player, Vehicle, VehicleState, WeaponArray, WeaponFire};
use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};

use log::debug;
use std::collections::HashMap;
use std::f32::consts::PI;

#[derive(SystemDesc)]
pub struct MoveWeaponFireSystem;


impl<'s> System<'s> for MoveWeaponFireSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WeaponFire>,
        ReadStorage<'s, Vehicle>,
        ReadStorage<'s, WeaponArray>,
        ReadStorage<'s, Player>,
        Read<'s, Time>,
    );

    fn run(
        &mut self,
        (entities, mut transforms, mut weapon_fires, vehicles, weapon_arrays, players, time): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        let mut vehicle_owner_map = HashMap::new();
        let mut heat_seeking_angle_map = HashMap::new();

        for (entity, weapon_fire, transform) in (&entities, &mut weapon_fires, &transforms).join() {
            weapon_fire.shot_life_timer += dt;

            if weapon_fire.shot_life_limit >= 0.0
                && weapon_fire.shot_life_timer >= weapon_fire.shot_life_limit
            {
                let _ = entities.delete(entity);
            } else {
                let fire_x = transform.translation().x;
                let fire_y = transform.translation().y;

                if weapon_fire.heat_seeking {
                    let mut closest_vehicle_x_diff = 0.0;
                    let mut closest_vehicle_y_diff = 0.0;
                    let mut closest_vehicle_dist = 1_000_000_000.0;

                    for (vehicle, vehicle_transform, player) in
                        (&vehicles, &transforms, &players).join()
                    {
                        if vehicle.state == VehicleState::Active {
                            if weapon_fire.owner_player_id != player.id {
                                let vehicle_x = vehicle_transform.translation().x;
                                let vehicle_y = vehicle_transform.translation().y;

                                // let weapon_rotation = transform.rotation();
                                // let (_, _, weapon_angle) = weapon_rotation.euler_angles();

                                let dist = ((vehicle_x - fire_x).powi(2)
                                    + (vehicle_y - fire_y).powi(2))
                                .sqrt();

                                if dist < closest_vehicle_dist {
                                    closest_vehicle_dist = dist;
                                    closest_vehicle_x_diff = fire_x - vehicle_x;
                                    closest_vehicle_y_diff = fire_y - vehicle_y;
                                }
                            }
                        }
                    }

                    let target_angle =
                        closest_vehicle_y_diff.atan2(closest_vehicle_x_diff) + (PI / 2.0); //rotate by PI/2 to line up with yaw angle
                                                                                           //let velocity_angle = weapon_fire.dy.atan2(weapon_fire.dx) + (PI / 2.0);

                    heat_seeking_angle_map.insert(entity.id(), target_angle);
                }

                if weapon_fire.attached {
                    for (_vehicle, vehicle_transform, weapon_array, player) in
                        (&vehicles, &transforms, &weapon_arrays, &players).join()
                    {
                        if weapon_fire.owner_player_id == player.id {
                            let vehicle_rotation = vehicle_transform.rotation();
                            let (_, _, yaw) = vehicle_rotation.euler_angles();

                            for (weapon_idx, weapon) in weapon_array.weapons.iter().enumerate() {
                                if let Some(weapon) = weapon {

                                    //undeploy old attached weapons
                                    if weapon_fire.weapon_array_id == weapon_idx 
                                            && weapon.name != weapon_fire.weapon_name {
                                        weapon_fire.deployed = false;
                                    }

                                    if weapon.name == weapon_fire.weapon_name && weapon.stats.attached {
                                        //pass on deployed status
                                        if weapon.stats.deployed == false {
                                            weapon_fire.deployed = false;
                                            let _ = entities.delete(entity);
                                        }
                                        else if weapon.stats.deployed == true {
                                            weapon_fire.deployed = true;
                                        }
                                    }

                                    vehicle_owner_map.insert(
                                        weapon_fire.owner_player_id,
                                        (
                                            vehicle_transform.translation().x,
                                            vehicle_transform.translation().y,
                                            yaw,
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        for (entity, weapon_fire, transform) in
            (&*entities, &mut weapon_fires, &mut transforms).join()
        {
            if weapon_fire.active {
                if weapon_fire.heat_seeking {
                    let heat_seeking_data = heat_seeking_angle_map.get(&entity.id());

                    if let Some(heat_seeking_data) = heat_seeking_data {
                        let angle = heat_seeking_data;

                        transform.set_rotation_2d(*angle);

                        let velocity_x_comp = -angle.sin(); //left is -, right is +
                        let velocity_y_comp = angle.cos(); //up is +, down is -

                        let sq_vel = weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2);
                        let abs_vel = sq_vel.sqrt();

                        weapon_fire.dx += weapon_fire.heat_seeking_agility * velocity_x_comp * dt;
                        weapon_fire.dx *= weapon_fire.shot_speed / abs_vel;

                        weapon_fire.dy += weapon_fire.heat_seeking_agility * velocity_y_comp * dt;
                        weapon_fire.dy *= weapon_fire.shot_speed / abs_vel;
                    }
                }

                if weapon_fire.attached {
                    if weapon_fire.deployed {
                        let vehicle_owner_data = vehicle_owner_map.get(&weapon_fire.owner_player_id);

                        if let Some(vehicle_owner_data) = vehicle_owner_data {
                            let (x, y, vehicle_angle) = vehicle_owner_data;

                            let angle = vehicle_angle + weapon_fire.spawn_angle;

                            let yaw_x_comp = -angle.sin(); //left is -, right is +
                            let yaw_y_comp = angle.cos(); //up is +, down is -

                            debug!("attached: {}, {}, {}", x, y, angle);

                            transform.set_rotation_2d(angle - PI);
                            transform.set_translation_x(x + yaw_x_comp * 14.0);
                            transform.set_translation_y(y + yaw_y_comp * 14.0);
                        }
                    } else {
                        let _ = entities.delete(entity);
                    }
                } else { //move to updated position based on velocity
                    if weapon_fire.shot_speed > 0.0 {
                        if weapon_fire.accel_rate > 0.0 {
                            let sq_vel = weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2);
                            let abs_vel = sq_vel.sqrt();

                            let new_speed = abs_vel + weapon_fire.accel_rate * dt;
                            weapon_fire.shot_speed = new_speed;

                            let scalar = new_speed / abs_vel;
                            weapon_fire.dx *= scalar;
                            weapon_fire.dy *= scalar;
                        }

                        transform.prepend_translation_x(weapon_fire.dx * dt);
                        transform.prepend_translation_y(weapon_fire.dy * dt);

                        let fire_x = transform.translation().x;
                        let fire_y = transform.translation().y;

                        //out of arena logic
                        if (fire_x > (ARENA_WIDTH + 2.0 * weapon_fire.width))
                            || (fire_x < (-2.0 * weapon_fire.width))
                            || (fire_y > (ARENA_HEIGHT + 2.0 * weapon_fire.width))
                            || (fire_y < (UI_HEIGHT - 2.0 * weapon_fire.width))
                        {
                            if !weapon_fire.attached {
                                if weapon_fire.bounces > 0 {
                                    weapon_fire.bounces -= 1;

                                    if (fire_x > (ARENA_WIDTH + 2.0 * weapon_fire.width))
                                        || (fire_x < (-2.0 * weapon_fire.width)) 
                                    {
                                        weapon_fire.dx *= -1.0;

                                        let new_angle = weapon_fire.dy.atan2(weapon_fire.dx) + (PI / 2.0); 
                                        //rotate by PI/2 to line up with 0deg is pointed towards top
                                        
                                        transform.set_rotation_2d(new_angle);
                                    }
                                    else if (fire_y > (ARENA_HEIGHT + 2.0 * weapon_fire.width))
                                        || (fire_y < (UI_HEIGHT - 2.0 * weapon_fire.width))
                                    {
                                        weapon_fire.dy *= -1.0;

                                        let new_angle = weapon_fire.dy.atan2(weapon_fire.dx) + (PI / 2.0); 
                                        //rotate by PI/2 to line up with 0deg is pointed towards top
                                        
                                        transform.set_rotation_2d(new_angle);
                                    }
                                }
                                else {
                                    let _ = entities.delete(entity);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
