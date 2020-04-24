use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage, ReadExpect, Entities};
use amethyst::input::{InputHandler, StringBindings};

use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
};

use std::f32::consts::PI;

use crate::components::{Vehicle, Player, kill_restart_vehicle, check_respawn_vehicle};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT,
    vehicle_damage_model, BASE_COLLISION_DAMAGE, COLLISION_PIERCING_DAMAGE_PCT, COLLISION_SHIELD_DAMAGE_PCT,
    COLLISION_ARMOR_DAMAGE_PCT, COLLISION_HEALTH_DAMAGE_PCT};


use std::ops::Deref;
use crate::audio::{play_bounce_sound, Sounds};


#[derive(SystemDesc)]
pub struct VehicleMoveSystem;

impl<'s> System<'s> for VehicleMoveSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,  //<MovementBindingTypes>>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(&mut self, (mut players, mut transforms, mut vehicles, 
            time, input, storage, sounds, audio_output): Self::SystemData) {

        let dt = time.delta_seconds();

        for (player, vehicle, transform) in (&mut players, &mut vehicles, &mut transforms).join() {
            if vehicle.in_respawn == true {
                check_respawn_vehicle(vehicle, transform, dt);
            }
            else {
                //let vehicle_accel = input.axis_value(&AxisBinding::VehicleAccel(player.id));
                //let vehicle_turn = input.axis_value(&AxisBinding::VehicleTurn(player.id));

                let (vehicle_accel, vehicle_turn) = match player.id {
                    0 => (input.axis_value("p1_accel"), input.axis_value("p1_turn")),
                    1 => (input.axis_value("p2_accel"), input.axis_value("p2_turn")),
                    2 => (input.axis_value("p3_accel"), input.axis_value("p3_turn")),
                    3 => (input.axis_value("p4_accel"), input.axis_value("p4_turn")),
                    _ => (None, None)
                };

                //let max_velocity: f32 = 0.5;

                let rotate_accel_rate: f32 = 1.0 * vehicle.engine_power/100.0;
                let rotate_friction_decel_rate: f32 = 0.97 * vehicle.engine_power/100.0;

                let thrust_accel_rate: f32 = 0.9 * vehicle.engine_power/100.0;
                let thrust_decel_rate: f32 = 0.6 * vehicle.engine_power/100.0;
                let thrust_friction_decel_rate: f32 = 0.3 * vehicle.engine_power/100.0;

                let wall_hit_non_bounce_decel_pct: f32 = 0.35;
                let wall_hit_bounce_decel_pct: f32 = -wall_hit_non_bounce_decel_pct;

                

                //println!("accel_input:{}, turn_input:{}", vehicle_accel.unwrap(), vehicle_turn.unwrap());

                let vehicle_rotation = transform.rotation();

                let (_, _, yaw) = vehicle_rotation.euler_angles();

                //println!("yaw:{}", yaw);

                let yaw_x_comp = -yaw.sin(); //left is -, right is +
                let yaw_y_comp = yaw.cos(); //up is +, down is -

                //println!("yaw_x_comp:{0:>6.3}, yaw_y_comp:{1:>6.3}", yaw_x_comp, yaw_y_comp);

                //Update vehicle velocity from vehicle speed accel input
                if let Some(move_amount) = vehicle_accel {

                    let scaled_amount: f32 = if move_amount > 0.0 {
                        thrust_accel_rate * move_amount as f32
                    }
                    else {
                        thrust_decel_rate * move_amount as f32
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

                vehicle.dx -= thrust_friction_decel_rate * velocity_x_comp * dt;
                vehicle.dy -= thrust_friction_decel_rate * velocity_y_comp * dt;


                //println!("vel_x:{0:>6.3}, vel_y:{1:>6.3}", vehicle.dx, vehicle.dy);


                let sq_vel = vehicle.dx.powi(2) + vehicle.dy.powi(2);
                let abs_vel = sq_vel.sqrt();

                // if abs_vel > max_velocity {
                //     vehicle.dx = velocity_x_comp * max_velocity;
                //     vehicle.dy = velocity_y_comp * max_velocity;
                // }

                // println!("{}",abs_vel);


                //Transform on vehicle velocity
                transform.prepend_translation_x(vehicle.dx);

                transform.prepend_translation_y(vehicle.dy);



                //Apply vehicle rotation from turn input
                if let Some(turn_amount) = vehicle_turn {
                    let scaled_amount = rotate_accel_rate * turn_amount as f32;

                    if scaled_amount > 0.1 || scaled_amount < -0.1 {
                        if vehicle.dr > 0.01 {
                            vehicle.dr += (scaled_amount - rotate_friction_decel_rate) * dt;
                        }
                        else if vehicle.dr < -0.01 {
                            vehicle.dr += (scaled_amount + rotate_friction_decel_rate) * dt;
                        }
                        else {
                            vehicle.dr += (scaled_amount) * dt;
                        }   
                    }
                    else if vehicle.dr > 0.01 {
                        vehicle.dr += (-rotate_friction_decel_rate) * dt;
                    }
                    else if vehicle.dr < -0.01 {
                        vehicle.dr += (rotate_friction_decel_rate) * dt;
                    }
                    else {
                        vehicle.dr = 0.0;
                    }  
                    
                    vehicle.dr = vehicle.dr.min(0.025).max(-0.025);

                    transform.set_rotation_2d(yaw + vehicle.dr);
                }



                //Wall-collision logic
                let vehicle_x = transform.translation().x;
                let vehicle_y = transform.translation().y;

                let yaw_width = vehicle.height*0.5 * yaw_x_comp.abs() + vehicle.width*0.5 * (1.0-yaw_x_comp.abs());
                let yaw_height = vehicle.height*0.5 * yaw_y_comp.abs() + vehicle.width*0.5 * (1.0-yaw_y_comp.abs());

                let mut x_collision = false;
                let mut y_collision = false;
                
                if vehicle_x > (ARENA_WIDTH - yaw_width) { //hit the right wall
                    transform.set_translation_x(ARENA_WIDTH - yaw_width);
                    x_collision = true;
                }
                else if vehicle_x < (yaw_width) { //hit the left wall
                    transform.set_translation_x(yaw_width);
                    x_collision = true;
                }

                if vehicle_y > (ARENA_HEIGHT - yaw_height) { //hit the top wall
                    transform.set_translation_y(ARENA_HEIGHT - yaw_height);
                    y_collision = true;
                }
                else if vehicle_y < (UI_HEIGHT - yaw_height) { //hit the bottom wall
                    transform.set_translation_y(UI_HEIGHT - yaw_height);
                    y_collision = true;
                }

                if x_collision {
                    vehicle.dx *= wall_hit_bounce_decel_pct * velocity_x_comp.abs();
                    vehicle.dy *= wall_hit_non_bounce_decel_pct * velocity_y_comp.abs();

                    if vehicle.collision_cooldown_timer <= 0.0 {
                        let damage:f32 = BASE_COLLISION_DAMAGE * abs_vel * velocity_x_comp.abs();
                        //println!("Player {} has collided with {} damage", player.id, damage);

                        let vehicle_destroyed:bool = vehicle_damage_model(vehicle, damage, 
                            COLLISION_PIERCING_DAMAGE_PCT, COLLISION_SHIELD_DAMAGE_PCT,
                            COLLISION_ARMOR_DAMAGE_PCT, COLLISION_HEALTH_DAMAGE_PCT);

                        if vehicle_destroyed {
                            kill_restart_vehicle(vehicle, transform);
                        }

                        play_bounce_sound(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
                        vehicle.collision_cooldown_timer = 1.0;
                    }
                }
                if y_collision {
                    vehicle.dx *= wall_hit_non_bounce_decel_pct * velocity_x_comp.abs();
                    vehicle.dy *= wall_hit_bounce_decel_pct * velocity_y_comp.abs();

                    if vehicle.collision_cooldown_timer <= 0.0 {
                        let damage:f32 = BASE_COLLISION_DAMAGE * abs_vel * velocity_y_comp.abs();
                        //println!("Player {} has collided with {} damage", player.id, damage);

                        let vehicle_destroyed:bool = vehicle_damage_model(vehicle, damage, 
                            COLLISION_PIERCING_DAMAGE_PCT, COLLISION_SHIELD_DAMAGE_PCT,
                            COLLISION_ARMOR_DAMAGE_PCT, COLLISION_HEALTH_DAMAGE_PCT);
        
                        if vehicle_destroyed {
                            kill_restart_vehicle(vehicle, transform);
                        }

                        play_bounce_sound(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
                        vehicle.collision_cooldown_timer = 1.0;
                    }
                }

                vehicle.collision_cooldown_timer -= dt;
            }
        }
    }
}