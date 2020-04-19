use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage};
use amethyst::input::{InputHandler};

use std::f32::consts::PI;

use crate::rally::{Vehicle, Player, ARENA_HEIGHT, ARENA_WIDTH, AxisBinding, MovementBindingTypes};

pub const WALL_HIT_BOUNCE_DECEL_PCT: f32 = -0.35;
pub const WALL_HIT_NON_BOUNCE_DECEL_PCT: f32 = 0.35;

pub const VEHICLE_ROTATE_ACCEL_RATE: f32 = 3.2;
pub const VEHICLE_ACCEL_RATE: f32 = 0.9;
pub const VEHICLE_DECEL_RATE: f32 = 0.6;
pub const VEHICLE_FRICTION_DECEL_RATE: f32 = 0.3;


#[derive(SystemDesc)]
pub struct VehicleMoveSystem;

impl<'s> System<'s> for VehicleMoveSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        Read<'s, Time>,
        Read<'s, InputHandler<MovementBindingTypes>>,
    );

    fn run(&mut self, (mut players, mut transforms, mut vehicles, time, input): Self::SystemData) {
        for (player, vehicle, transform) in (&mut players, &mut vehicles, &mut transforms).join() {
            let vehicle_accel = input.axis_value(&AxisBinding::VehicleAccel(player.id));
            let vehicle_turn = input.axis_value(&AxisBinding::VehicleTurn(player.id));

            //println!("accel_input:{}, turn_input:{}", vehicle_accel.unwrap(), vehicle_turn.unwrap());

            let dt = time.delta_seconds();

            let vehicle_rotation = transform.rotation();

            let (_, _, yaw) = vehicle_rotation.euler_angles();

            //println!("yaw:{}", yaw);

            let yaw_x_comp = -yaw.sin(); //left is -, right is +
            let yaw_y_comp = yaw.cos(); //up is +, down is -

            //println!("yaw_x_comp:{0:>6.3}, yaw_y_comp:{1:>6.3}", yaw_x_comp, yaw_y_comp);

            //Update vehicle velocity from vehicle speed accel input
            if let Some(move_amount) = vehicle_accel {

                let scaled_amount: f32 = if move_amount > 0.0 {
                    VEHICLE_ACCEL_RATE * move_amount as f32
                }
                else {
                    VEHICLE_DECEL_RATE * move_amount as f32
                };

                vehicle.dx += scaled_amount * yaw_x_comp * dt;
                vehicle.dy += scaled_amount * yaw_y_comp * dt;
            }

            //println!("vel_x:{}, vel_y:{}", vehicle.dx, vehicle.dy);
            
            //Apply friction
            //this needs to be applied to vehicle momentum angle, not yaw angle
            let velocity_angle = vehicle.dy.atan2(vehicle.dx) - (PI/2.0); //rotate by PI/2 to line up with yaw angle

            //println!("vel_angle:{}", velocity_angle);

            let velocity_x_comp = -velocity_angle.sin(); //left is -, right is +
            let velocity_y_comp = velocity_angle.cos(); //up is +, down is -

            //println!("vel_angle_sin:{0:>6.3}, vel_angle_cos:{1:>6.3}", velocity_x_comp, velocity_y_comp);

            vehicle.dx -= VEHICLE_FRICTION_DECEL_RATE * velocity_x_comp * dt;
            vehicle.dy -= VEHICLE_FRICTION_DECEL_RATE * velocity_y_comp * dt;


            //println!("vel_x:{0:>6.3}, vel_y:{1:>6.3}", vehicle.dx, vehicle.dy);


            //Transform on vehicle velocity
            transform.prepend_translation_x(vehicle.dx);

            transform.prepend_translation_y(vehicle.dy);



            //Apply vehicle rotation from turn input
            if let Some(turn_amount) = vehicle_turn {
                let scaled_amount = VEHICLE_ROTATE_ACCEL_RATE * turn_amount as f32;

                transform.set_rotation_2d(yaw + (scaled_amount * dt));
            }


            //Wall-collision logic
            let vehicle_x = transform.translation().x;
            let vehicle_y = transform.translation().y;

            let yaw_width = vehicle.height*0.5 * yaw_x_comp.abs() + vehicle.width*0.5 * (1.0-yaw_x_comp.abs());
            let yaw_height = vehicle.height*0.5 * yaw_y_comp.abs() + vehicle.width*0.5 * (1.0-yaw_y_comp.abs());

            if vehicle_x > (ARENA_WIDTH - yaw_width) { //hit the right wall
                transform.set_translation_x(ARENA_WIDTH - yaw_width);
                vehicle.dx *= WALL_HIT_BOUNCE_DECEL_PCT * velocity_x_comp.abs();
                vehicle.dy *= WALL_HIT_NON_BOUNCE_DECEL_PCT * velocity_y_comp.abs();
            }
            else if vehicle_x < (yaw_width) { //hit the left wall
                transform.set_translation_x(yaw_width);
                vehicle.dx *= WALL_HIT_BOUNCE_DECEL_PCT * velocity_x_comp.abs();
                vehicle.dy *= WALL_HIT_NON_BOUNCE_DECEL_PCT * velocity_y_comp.abs();
            }

            if vehicle_y > (ARENA_HEIGHT - yaw_height) { //hit the top wall
                transform.set_translation_y(ARENA_HEIGHT - yaw_height);
                vehicle.dx *= WALL_HIT_NON_BOUNCE_DECEL_PCT * velocity_x_comp.abs();
                vehicle.dy *= WALL_HIT_BOUNCE_DECEL_PCT * velocity_y_comp.abs();
            }
            else if vehicle_y < (yaw_height) { //hit the bottom wall
                transform.set_translation_y(yaw_height);
                vehicle.dx *= WALL_HIT_NON_BOUNCE_DECEL_PCT * velocity_x_comp.abs();
                vehicle.dy *= WALL_HIT_BOUNCE_DECEL_PCT * velocity_y_comp.abs();
            }
        }
    }
}