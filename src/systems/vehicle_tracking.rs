use amethyst::core::{Transform};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, ReadStorage, System, SystemData, WriteStorage};

use std::f32::consts::PI;
use std::collections::HashMap;

use crate::components::{
    Player, Vehicle,
};


#[derive(SystemDesc)]
pub struct VehicleTrackingSystem;

impl<'s> System<'s> for VehicleTrackingSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
    );

    fn run(
        &mut self,
        (
            mut players,
            transforms,
            mut vehicles,
        ): Self::SystemData,
    ) {
        //Find closest target
        let mut closest_target_angles_map = HashMap::new();

        for (player1, _vehicle1, vehicle1_transform) in (&players, &vehicles, &transforms).join() {
            let mut closest_vehicle_x_diff = 0.0;
            let mut closest_vehicle_y_diff = 0.0;
            let mut closest_vehicle_dist = 1_000_000_000.0;

            let vehicle1_x = vehicle1_transform.translation().x;
            let vehicle1_y = vehicle1_transform.translation().y;

            for (player2, _vehicle2, vehicle2_transform) in
                (&players, &vehicles, &transforms).join()
            {
                if player1.id != player2.id {
                    let vehicle2_x = vehicle2_transform.translation().x;
                    let vehicle2_y = vehicle2_transform.translation().y;

                    let dist = ((vehicle2_x - vehicle1_x).powi(2)
                        + (vehicle2_y - vehicle1_y).powi(2))
                    .sqrt();

                    if dist < closest_vehicle_dist {
                        closest_vehicle_dist = dist;
                        closest_vehicle_x_diff = vehicle1_x - vehicle2_x;
                        closest_vehicle_y_diff = vehicle1_y - vehicle2_y;
                    }
                }
            }

            let mut target_angle =
                closest_vehicle_y_diff.atan2(closest_vehicle_x_diff) + (PI / 2.0); //rotate by PI/2 to line up with yaw angle
            if target_angle > PI {
                target_angle -= 2.0 * PI;
            }

            closest_target_angles_map.insert(player1.id,
                (target_angle, closest_vehicle_dist));
        }

        //Assign Tracking Data
        for (player, vehicle) in (&mut players, &mut vehicles).join()
        {
            let closest_target_angles = closest_target_angles_map.get(&player.id);

            if let Some(closest_target_angles) = closest_target_angles {
                let (target_angle, closest_vehicle_dist) = closest_target_angles;

                vehicle.angle_to_closest_vehicle = *target_angle;
                vehicle.dist_to_closest_vehicle = *closest_vehicle_dist;
            }
        }
    }
}
