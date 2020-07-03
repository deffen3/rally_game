use amethyst::core::Transform;
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, ReadStorage, System, SystemData, WriteStorage};

use std::collections::HashMap;
use std::f32::consts::PI;

use crate::components::{Player, Vehicle, VehicleState, WeaponArray};

#[derive(SystemDesc)]
pub struct VehicleTrackingSystem;

impl<'s> System<'s> for VehicleTrackingSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        ReadStorage<'s, WeaponArray>,
    );

    fn run(&mut self, (mut players, transforms, mut vehicles, weapon_arrays): Self::SystemData) {
        //Find closest vehicle, and closest targetable vehicle within weapon angle target range
        let mut closest_target_angles_map = HashMap::new();

        for (player1, _vehicle1, vehicle1_transform, weapon_array) in
            (&players, &vehicles, &transforms, &weapon_arrays).join()
        {

            if weapon_array.installed.len() > 0 {
                let primary_weapon = &weapon_array.installed[0].weapon;

                let mut closest_vehicle_dist: Option<f32> = None;
                let mut closest_vehicle_target_angle: Option<f32> = None;

                let mut closest_targetable_vehicle_dist: Option<f32> = None;
                let mut closest_targetable_vehicle_target_angle: Option<f32> = None;

                let vehicle1_x = vehicle1_transform.translation().x;
                let vehicle1_y = vehicle1_transform.translation().y;

                let (_, _, vehicle_angle) = vehicle1_transform.rotation().euler_angles();

                let install_mounted_angle;
                if weapon_array.installed[0].mounted_angle.is_none() {
                    install_mounted_angle = 0.0;
                }
                else {
                    install_mounted_angle = weapon_array.installed[0].mounted_angle.unwrap();
                }

                //typical angle this weapon should fire at
                let standard_angle = vehicle_angle + install_mounted_angle + 
                    primary_weapon.stats.fire_stats.mount_angle_special_offset;

                for (player2, vehicle2, vehicle2_transform) in (&players, &vehicles, &transforms).join()
                {
                    //Other player must be active and not on the same team
                    if player1.id != player2.id && 
                            player1.team != player2.team &&
                            vehicle2.state == VehicleState::Active {
                        let vehicle2_x = vehicle2_transform.translation().x;
                        let vehicle2_y = vehicle2_transform.translation().y;

                        let dist = ((vehicle2_x - vehicle1_x).powi(2)
                            + (vehicle2_y - vehicle1_y).powi(2))
                        .sqrt();

                        let x_diff = vehicle1_x - vehicle2_x;
                        let y_diff = vehicle1_y - vehicle2_y;

                        let mut target_angle = y_diff.atan2(x_diff) + (PI / 2.0); //rotate by PI/2 to line up with 0deg is pointed towards top

                        if target_angle > PI {
                            target_angle -= 2.0 * PI;
                        }

                        //Save closest vehicle, ...
                        if closest_vehicle_dist.is_none() || dist < closest_vehicle_dist.unwrap() {
                            closest_vehicle_dist = Some(dist);
                            closest_vehicle_target_angle = Some(target_angle);
                        }

                        //...and closest targetable vehicle
                        let mut angle_diff = standard_angle - target_angle;

                        while angle_diff > PI || angle_diff < -PI {
                            if angle_diff > PI {
                                angle_diff = -(2.0 * PI - angle_diff);
                            } else if angle_diff < -PI {
                                angle_diff = -(-2.0 * PI - angle_diff);
                            }
                        }

                        let targetable;
                        if angle_diff.abs() < primary_weapon.stats.tracking_angle {
                            targetable = true;
                        } else {
                            targetable = false;
                        }

                        if targetable {
                            if closest_targetable_vehicle_dist.is_none()
                                || dist < closest_targetable_vehicle_dist.unwrap()
                            {
                                closest_targetable_vehicle_dist = Some(dist);
                                closest_targetable_vehicle_target_angle = Some(target_angle);
                            }
                        }
                    }
                }

                closest_target_angles_map.insert(
                    player1.id,
                    (
                        closest_vehicle_target_angle,
                        closest_vehicle_dist,
                        closest_targetable_vehicle_target_angle,
                        closest_targetable_vehicle_dist,
                    ),
                );
            }
        }

        //Assign Tracking Data
        for (player, vehicle) in (&mut players, &mut vehicles).join() {
            let closest_target_angles = closest_target_angles_map.get(&player.id);

            if let Some(closest_target_angles) = closest_target_angles {
                let (
                    closest_vehicle_target_angle,
                    closest_vehicle_dist,
                    closest_targetable_vehicle_target_angle,
                    closest_targetable_vehicle_dist,
                ) = closest_target_angles;

                vehicle.angle_to_closest_vehicle = *closest_vehicle_target_angle;
                vehicle.dist_to_closest_vehicle = *closest_vehicle_dist;

                vehicle.angle_to_closest_targetable_vehicle =
                    *closest_targetable_vehicle_target_angle;
                vehicle.dist_to_closest_targetable_vehicle = *closest_targetable_vehicle_dist;
            }
        }
    }
}
