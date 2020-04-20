use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage, ReadStorage, Entities, Write};

use std::f32::consts::PI;

use crate::rally::{Vehicle, Player};

#[derive(SystemDesc, Default)]
pub struct CollisionVehToVehSystem;


impl<'s> System<'s> for CollisionVehToVehSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, transforms, players, mut vehicles, time): Self::SystemData) {
        //let dt = time.delta_seconds();

        for (vehicle_1_entity, vehicle_1, player_1, vehicle_1_transform) in (&*entities, &vehicles, &players, &transforms).join() {
            let vehicle_1_x = vehicle_1_transform.translation().x;
            let vehicle_1_y = vehicle_1_transform.translation().y;

            for (vehicle_2_entity, vehicle_2, player_2, vehicle_2_transform) in (&*entities, &vehicles, &players, &transforms).join() {
                let vehicle_2_x = vehicle_2_transform.translation().x;
                let vehicle_2_y = vehicle_2_transform.translation().y;

                if player_1.id != player_2.id {
                    if (vehicle_1_x - vehicle_2_x).powi(2) + (vehicle_1_y - vehicle_2_y).powi(2) < vehicle_1.width.powi(2) {

                        let velocity_1_angle = vehicle_1.dy.atan2(vehicle_1.dx) - (PI/2.0); //rotate by PI/2 to line up with yaw angle
                        let velocity_1_x_comp = -velocity_1_angle.sin(); //left is -, right is +
                        let velocity_1_y_comp = velocity_1_angle.cos(); //up is +, down is -

                        //vehicle_1.dx *= VEHICLE_HIT_BOUNCE_DECEL_PCT * velocity_1_x_comp.abs();
                        //vehicle_1.dy *= VEHICLE_HIT_BOUNCE_DECEL_PCT * velocity_1_y_comp.abs();

                        let velocity_2_angle = vehicle_2.dy.atan2(vehicle_2.dx) - (PI/2.0); //rotate by PI/2 to line up with yaw angle
                        let velocity_2_x_comp = -velocity_2_angle.sin(); //left is -, right is +
                        let velocity_2_y_comp = velocity_2_angle.cos(); //up is +, down is -

                        //vehicle_2.dx *= VEHICLE_HIT_BOUNCE_DECEL_PCT * velocity_2_x_comp.abs();
                        //vehicle_2.dy *= VEHICLE_HIT_BOUNCE_DECEL_PCT * velocity_2_y_comp.abs();

                    }
                }
            }
        }
    }
}