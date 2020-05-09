use amethyst::core::{Transform};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, ReadStorage, System, SystemData, WriteStorage};

use std::f32::consts::PI;
use std::collections::HashMap;

use crate::components::{
    Player, Vehicle, VehicleState, Weapon
};


#[derive(SystemDesc)]
pub struct VehicleTrackingSystem;

impl<'s> System<'s> for VehicleTrackingSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        ReadStorage<'s, Weapon>,
    );

    fn run(
        &mut self,
        (
            mut players,
            transforms,
            mut vehicles,
            weapons,
        ): Self::SystemData,
    ) {
        //Find closest vehicle, and closest targetable vehicle within weapon angle target range
        let mut closest_target_angles_map = HashMap::new();

        for (player1, _vehicle1, vehicle1_transform, weapon) in (&players, &vehicles, &transforms, &weapons).join() {
            let mut closest_vehicle_dist: Option<f32> = None;
            let mut closest_vehicle_target_angle: Option<f32> = None;

            let mut closest_targetable_vehicle_dist: Option<f32> = None;
            let mut closest_targetable_vehicle_target_angle: Option<f32> = None;


            let vehicle1_x = vehicle1_transform.translation().x;
            let vehicle1_y = vehicle1_transform.translation().y;

            let (_, _, vehicle_angle) = vehicle1_transform.rotation().euler_angles();
        
            //typical angle this weapon should fire at
            let standard_angle = vehicle_angle + weapon.stats.mounted_angle;



            for (player2, vehicle2, vehicle2_transform) in
                (&players, &vehicles, &transforms).join()
            {
                if player1.id != player2.id && vehicle2.state == VehicleState::Active {
                    let vehicle2_x = vehicle2_transform.translation().x;
                    let vehicle2_y = vehicle2_transform.translation().y;

                    let dist = ((vehicle2_x - vehicle1_x).powi(2)
                        + (vehicle2_y - vehicle1_y).powi(2))
                    .sqrt();

                    let x_diff = vehicle1_x - vehicle2_x;
                    let y_diff = vehicle1_y - vehicle2_y;

                    let mut target_angle =
                        y_diff.atan2(x_diff) + (PI / 2.0); //rotate by PI/2 to line up with 0deg is pointed towards top
                        
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

                    if angle_diff > PI {
                        angle_diff = -(2.0*PI - angle_diff);
                    } else if angle_diff < -PI {
                        angle_diff = -(-2.0*PI - angle_diff);
                    }
                    
                    let targetable;
                    if angle_diff.abs() < weapon.stats.tracking_angle {
                        targetable = true;
                    } else {
                        targetable = false;
                    }

                    if targetable {
                        if closest_targetable_vehicle_dist.is_none() || dist < closest_targetable_vehicle_dist.unwrap() {
                            closest_targetable_vehicle_dist = Some(dist);
                            closest_targetable_vehicle_target_angle = Some(target_angle);
                        }
                    }
                }
            }

            closest_target_angles_map.insert(player1.id,
                (closest_vehicle_target_angle, closest_vehicle_dist,
                 closest_targetable_vehicle_target_angle, closest_targetable_vehicle_dist));
        }

        //Assign Tracking Data
        for (player, vehicle) in (&mut players, &mut vehicles).join()
        {
            let closest_target_angles = closest_target_angles_map.get(&player.id);

            if let Some(closest_target_angles) = closest_target_angles {
                let (closest_vehicle_target_angle, closest_vehicle_dist,
                    closest_targetable_vehicle_target_angle, closest_targetable_vehicle_dist) = closest_target_angles;

                vehicle.angle_to_closest_vehicle = *closest_vehicle_target_angle;
                vehicle.dist_to_closest_vehicle = *closest_vehicle_dist;

                vehicle.angle_to_closest_targetable_vehicle = *closest_targetable_vehicle_target_angle;
                vehicle.dist_to_closest_targetable_vehicle = *closest_targetable_vehicle_dist;
            }
        }
    }
}
