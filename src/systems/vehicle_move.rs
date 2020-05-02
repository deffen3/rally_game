use amethyst::core::{Time, Transform};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage};
use amethyst::input::{InputHandler, StringBindings};

use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
};

use log::debug;
use rand::Rng;
use std::f32::consts::PI;

use crate::components::{
    check_respawn_vehicle, kill_restart_vehicle, BotMode, Hitbox, Player, Vehicle, Weapon,
};

use crate::rally::{
    vehicle_damage_model, ARENA_HEIGHT, ARENA_WIDTH, BASE_COLLISION_DAMAGE,
    COLLISION_ARMOR_DAMAGE_PCT, COLLISION_HEALTH_DAMAGE_PCT, COLLISION_PIERCING_DAMAGE_PCT,
    COLLISION_SHIELD_DAMAGE_PCT, UI_HEIGHT,
};

use crate::audio::{play_bounce_sound, Sounds};


const BOT_COLLISION_TURN_COOLDOWN_RESET: f32 = 0.7;
const BOT_COLLISION_MOVE_COOLDOWN_RESET: f32 = 0.7;

const BOT_ENGAGE_DISTANCE: f32 = 160.0;
const BOT_DISENGAGE_DISTANCE: f32 = 240.0;


#[derive(SystemDesc)]
pub struct VehicleMoveSystem;

impl<'s> System<'s> for VehicleMoveSystem {
    type SystemData = (
        ReadStorage<'s, Hitbox>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        ReadStorage<'s, Weapon>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (
            hitboxes,
            mut players,
            mut transforms,
            mut vehicles,
            weapons,
            time,
            input,
            storage,
            sounds,
            audio_output,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        let dt = time.delta_seconds();

        //Turn and Accel
        for (player, vehicle, transform, weapon) in
            (&mut players, &mut vehicles, &mut transforms, &weapons).join()
        {
            if vehicle.in_respawn {
                check_respawn_vehicle(vehicle, transform, dt);
            } else {
                //let max_velocity: f32 = 0.5;

                let rotate_accel_rate: f32 = 1.0 * vehicle.engine_power / 100.0;
                let rotate_friction_decel_rate: f32 = 0.98 * vehicle.engine_power / 100.0;

                let thrust_accel_rate: f32 = 0.9 * vehicle.engine_power / 100.0;
                let thrust_decel_rate: f32 = 0.6 * vehicle.engine_power / 100.0;
                let thrust_friction_decel_rate: f32 = 0.3 * vehicle.engine_power / 100.0;

                let wall_hit_non_bounce_decel_pct: f32 = 0.35;
                let wall_hit_bounce_decel_pct: f32 = -wall_hit_non_bounce_decel_pct;

                //let vehicle_accel = input.axis_value(&AxisBinding::VehicleAccel(player.id));
                //let vehicle_turn = input.axis_value(&AxisBinding::VehicleTurn(player.id));

                let (mut vehicle_accel, mut vehicle_turn) = match player.id {
                    0 => (input.axis_value("p1_accel"), input.axis_value("p1_turn")),
                    1 => (input.axis_value("p2_accel"), input.axis_value("p2_turn")),
                    2 => (input.axis_value("p3_accel"), input.axis_value("p3_turn")),
                    3 => (input.axis_value("p4_accel"), input.axis_value("p4_turn")),
                    _ => (None, None),
                };

                let vehicle_rotation = transform.rotation();
                let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();

                player.bot_move_cooldown -= dt;

                if player.is_bot {
                    if player.bot_mode == BotMode::Running || player.bot_mode == BotMode::Mining {

                        if vehicle.dist_to_closest_vehicle <= BOT_ENGAGE_DISTANCE && player.bot_move_cooldown < 0.0
                        {
                            //change modes to attack

                            if weapon.stats.attached { //Typically just LaserSword
                                player.bot_mode = BotMode::Swording;
                                debug!("{} Swording", player.id);
                                player.bot_move_cooldown = 5.0;
                            } else if weapon.stats.shot_speed <= 0.0 { //Typically just Mines
                                player.bot_mode = BotMode::Mining;
                                debug!("{} Mining", player.id);
                            } else {
                                player.bot_mode = BotMode::StopAim;
                                debug!("{} StopAim", player.id);
                                player.bot_move_cooldown = 5.0;
                            }
                        } 
                        
                        if player.bot_mode == BotMode::Running || player.bot_mode == BotMode::Mining {
                            //continue with Running or Mining mode
                            if player.bot_move_cooldown < 0.0 {
                                //issue new move command
                                vehicle_accel = Some(rng.gen_range(0.2, 0.6) as f32);
                                vehicle_turn = Some(rng.gen_range(-1.0, 1.0) as f32);

                                player.last_accel_input = vehicle_accel;
                                player.last_turn_input = vehicle_turn;

                                player.bot_move_cooldown = player.bot_move_cooldown_reset;
                            } else {
                                //hold previous Running move
                                vehicle_accel = player.last_accel_input;
                                vehicle_turn = player.last_turn_input;
                            }
                        }
                    } else if player.bot_mode == BotMode::StopAim
                        || player.bot_mode == BotMode::Chasing
                        || player.bot_mode == BotMode::Swording
                    {
                        if vehicle.dist_to_closest_vehicle > BOT_DISENGAGE_DISTANCE || player.bot_move_cooldown < 0.0 {
                            player.bot_move_cooldown = player.bot_move_cooldown_reset;

                            let run_or_chase = rng.gen::<bool>();

                            if run_or_chase {
                                player.bot_mode = BotMode::Running;
                                debug!("{} Running", player.id);
                            } else {
                                player.bot_mode = BotMode::Chasing;
                                debug!("{} Chasing", player.id);
                            }
                        } else {
                            //continue with Attacking mode
                            let attack_angle;
                            let turn_value;

                            //Prepare magnitude of Turning and Acceleration input
                            if player.bot_mode == BotMode::Swording {
                                //spin target angle by 180deg for Sword Mode
                                if vehicle.angle_to_closest_vehicle < 0.0 {
                                    attack_angle = vehicle.angle_to_closest_vehicle + PI;
                                } else {
                                    attack_angle = vehicle.angle_to_closest_vehicle - PI;
                                }
                                turn_value = 1.0;
                                vehicle_accel = Some(-1.0);
                            } else if player.bot_mode == BotMode::Chasing {
                                attack_angle = vehicle.angle_to_closest_vehicle;
                                turn_value = 1.0;
                                vehicle_accel = Some(0.6);
                            } else {
                                attack_angle = vehicle.angle_to_closest_vehicle;
                                turn_value = 1.0;
                            }

                            //Solve for Angle and Direction to turn
                            let mut angle_diff = vehicle_angle - attack_angle;

                            if angle_diff > PI {
                                angle_diff = -(2.0*PI - angle_diff);
                            } else if angle_diff < -PI {
                                angle_diff = -(-2.0*PI - angle_diff);
                            }

                            if angle_diff > 0.001 {
                                vehicle_turn = Some(-turn_value);
                            } else if angle_diff > 0.001 {
                                vehicle_turn = Some(turn_value);
                            } else {
                                vehicle_turn = Some(0.0);
                            }
                        }
                    } else if player.bot_mode == BotMode::CollisionTurn {
                        vehicle_accel = Some(0.5);
                        vehicle_turn = Some(1.0);

                        if player.bot_move_cooldown < 0.0 {
                            player.bot_mode = BotMode::CollisionMove;
                            debug!("{} CollisionMove", player.id);
                            player.bot_move_cooldown = BOT_COLLISION_MOVE_COOLDOWN_RESET;
                        }
                    } else if player.bot_mode == BotMode::CollisionMove {
                        vehicle_accel = Some(0.5);
                        vehicle_turn = Some(0.0);

                        if player.bot_move_cooldown < 0.0 {
                            player.bot_mode = BotMode::Running;
                            debug!("{} Running", player.id);
                        }
                    }
                }

                debug!("accel_input:{}, turn_input:{}", vehicle_accel.unwrap(), vehicle_turn.unwrap());

                if player.id == 0 {
                    debug!("vehicle_angle:{}", vehicle_angle);
                }

                let yaw_x_comp = -vehicle_angle.sin(); //left is -, right is +
                let yaw_y_comp = vehicle_angle.cos(); //up is +, down is -

                debug!("yaw_x_comp:{0:>6.3}, yaw_y_comp:{1:>6.3}", yaw_x_comp, yaw_y_comp);

                //Update vehicle velocity from vehicle speed accel input
                if let Some(move_amount) = vehicle_accel {
                    let scaled_amount: f32 = if vehicle.repair.activated {
                        0.0 as f32
                    } else if move_amount > 0.0 {
                        thrust_accel_rate * move_amount as f32
                    } else {
                        thrust_decel_rate * move_amount as f32
                    };

                    vehicle.dx += scaled_amount * yaw_x_comp * dt;
                    vehicle.dy += scaled_amount * yaw_y_comp * dt;
                }

                debug!("vel_x:{}, vel_y:{}", vehicle.dx, vehicle.dy);

                //Apply friction
                //this needs to be applied to vehicle momentum angle, not vehicle_angle angle
                let velocity_angle = vehicle.dy.atan2(vehicle.dx) - (PI / 2.0); //rotate by PI/2 to line up with vehicle_angle angle

                debug!("vel_angle:{}", velocity_angle);

                let velocity_x_comp = -velocity_angle.sin(); //left is -, right is +
                let velocity_y_comp = velocity_angle.cos(); //up is +, down is -

                debug!("vel_angle_sin:{0:>6.3}, vel_angle_cos:{1:>6.3}", velocity_x_comp, velocity_y_comp);

                vehicle.dx -= thrust_friction_decel_rate * velocity_x_comp * dt;
                vehicle.dy -= thrust_friction_decel_rate * velocity_y_comp * dt;

                debug!("vel_x:{0:>6.3}, vel_y:{1:>6.3}", vehicle.dx, vehicle.dy);

                let sq_vel = vehicle.dx.powi(2) + vehicle.dy.powi(2);
                let abs_vel = sq_vel.sqrt();

                // if abs_vel > max_velocity {
                //     vehicle.dx = velocity_x_comp * max_velocity;
                //     vehicle.dy = velocity_y_comp * max_velocity;
                // }

                debug!("{}",abs_vel);

                //Transform on vehicle velocity
                transform.prepend_translation_x(vehicle.dx);

                transform.prepend_translation_y(vehicle.dy);

                //Apply vehicle rotation from turn input
                if let Some(turn_amount) = vehicle_turn {
                    let scaled_amount: f32 = if vehicle.repair.activated == true {
                        0.0 as f32
                    }
                    else {
                        rotate_accel_rate * turn_amount as f32
                    };

                    if scaled_amount > 0.1 || scaled_amount < -0.1 {
                        if vehicle.dr > 0.01 {
                            vehicle.dr += (scaled_amount - rotate_friction_decel_rate) * dt;
                        } else if vehicle.dr < -0.01 {
                            vehicle.dr += (scaled_amount + rotate_friction_decel_rate) * dt;
                        } else {
                            vehicle.dr += (scaled_amount) * dt;
                        }
                    } else if vehicle.dr > 0.01 {
                        vehicle.dr += (-rotate_friction_decel_rate) * dt;
                    } else if vehicle.dr < -0.01 {
                        vehicle.dr += (rotate_friction_decel_rate) * dt;
                    } else {
                        vehicle.dr = 0.0;
                    }

                    vehicle.dr = vehicle.dr.min(0.025).max(-0.025);

                    transform.set_rotation_2d(vehicle_angle + vehicle.dr);
                }

                //Wall-collision logic
                let vehicle_x = transform.translation().x;
                let vehicle_y = transform.translation().y;

                let yaw_width = vehicle.height * 0.5 * yaw_x_comp.abs()
                    + vehicle.width * 0.5 * (1.0 - yaw_x_comp.abs());
                let yaw_height = vehicle.height * 0.5 * yaw_y_comp.abs()
                    + vehicle.width * 0.5 * (1.0 - yaw_y_comp.abs());

                let mut x_collision = false;
                let mut y_collision = false;

                if vehicle_x > (ARENA_WIDTH - yaw_width) {
                    //hit the right wall
                    transform.set_translation_x(ARENA_WIDTH - yaw_width);
                    x_collision = true;
                } else if vehicle_x < (yaw_width) {
                    //hit the left wall
                    transform.set_translation_x(yaw_width);
                    x_collision = true;
                }

                if vehicle_y > (ARENA_HEIGHT - yaw_height) {
                    //hit the top wall
                    transform.set_translation_y(ARENA_HEIGHT - yaw_height);
                    y_collision = true;
                } else if vehicle_y < (UI_HEIGHT + yaw_height) {
                    //hit the bottom wall
                    transform.set_translation_y(UI_HEIGHT + yaw_height);
                    y_collision = true;
                }

                if x_collision {
                    vehicle.dx *= wall_hit_bounce_decel_pct * velocity_x_comp.abs();
                    vehicle.dy *= wall_hit_non_bounce_decel_pct * velocity_y_comp.abs();

                    if vehicle.collision_cooldown_timer <= 0.0 {
                        let damage: f32 = BASE_COLLISION_DAMAGE * abs_vel * velocity_x_comp.abs();
                        debug!("Player {} has collided with {} damage", player.id, damage);

                        let vehicle_destroyed: bool = vehicle_damage_model(
                            vehicle,
                            damage,
                            COLLISION_PIERCING_DAMAGE_PCT,
                            COLLISION_SHIELD_DAMAGE_PCT,
                            COLLISION_ARMOR_DAMAGE_PCT,
                            COLLISION_HEALTH_DAMAGE_PCT,
                        );

                        if vehicle_destroyed {
                            kill_restart_vehicle(vehicle, transform);
                        }

                        if abs_vel > 0.5 {
                            play_bounce_sound(
                                &*sounds,
                                &storage,
                                audio_output.as_deref(),
                            );
                        }
                        vehicle.collision_cooldown_timer = 1.0;
                    }
                }
                if y_collision {
                    vehicle.dx *= wall_hit_non_bounce_decel_pct * velocity_x_comp.abs();
                    vehicle.dy *= wall_hit_bounce_decel_pct * velocity_y_comp.abs();

                    if vehicle.collision_cooldown_timer <= 0.0 {
                        let damage: f32 = BASE_COLLISION_DAMAGE * abs_vel * velocity_y_comp.abs();
                        debug!("Player {} has collided with {} damage", player.id, damage);

                        let vehicle_destroyed: bool = vehicle_damage_model(
                            vehicle,
                            damage,
                            COLLISION_PIERCING_DAMAGE_PCT,
                            COLLISION_SHIELD_DAMAGE_PCT,
                            COLLISION_ARMOR_DAMAGE_PCT,
                            COLLISION_HEALTH_DAMAGE_PCT,
                        );

                        if vehicle_destroyed {
                            kill_restart_vehicle(vehicle, transform);
                        }

                        if abs_vel > 0.5 {
                            play_bounce_sound(
                                &*sounds,
                                &storage,
                                audio_output.as_deref(),
                            );
                        }
                        vehicle.collision_cooldown_timer = 1.0;
                    }
                }

                if player.is_bot
                    && (x_collision || y_collision)
                    && (player.bot_mode != BotMode::CollisionTurn)
                    && (player.bot_mode != BotMode::CollisionMove)
                {
                    player.bot_mode = BotMode::CollisionTurn;
                    debug!("{} CollisionTurn", player.id);
                    player.bot_move_cooldown = BOT_COLLISION_TURN_COOLDOWN_RESET;
                }

                vehicle.collision_cooldown_timer -= dt;
            }
        }

        //hitbox collision logic, right now only applied to circular arena obstacles
        let mut player_destroyed: Vec<usize> = Vec::new();
        let mut player_arena_bounce: Vec<usize> = Vec::new();

        for (hitbox, hitbox_transform) in (&hitboxes, &transforms).join() {
            let hitbox_x = hitbox_transform.translation().x;
            let hitbox_y = hitbox_transform.translation().y;

            for (player, vehicle, transform) in (&mut players, &mut vehicles, &transforms).join() {
                let wall_hit_non_bounce_decel_pct: f32 = 0.65;
                let wall_hit_bounce_decel_pct: f32 = -wall_hit_non_bounce_decel_pct;

                let vehicle_x = transform.translation().x;
                let vehicle_y = transform.translation().y;

                let sq_vel = vehicle.dx.powi(2) + vehicle.dy.powi(2);
                let abs_vel = sq_vel.sqrt();

                if (vehicle_x - hitbox_x).powi(2) + (vehicle_y - hitbox_y).powi(2)
                    < (hitbox.width / 2.0 + vehicle.width / 2.0).powi(2)
                {
                    vehicle.dx *= wall_hit_bounce_decel_pct;
                    vehicle.dy *= wall_hit_bounce_decel_pct;

                    player_arena_bounce.push(player.id.clone());

                    if player.is_bot
                        && (player.bot_mode != BotMode::CollisionTurn)
                        && (player.bot_mode != BotMode::CollisionMove)
                    {
                        player.bot_mode = BotMode::CollisionTurn;
                        debug!("{} CollisionTurn", player.id);
                        player.bot_move_cooldown = BOT_COLLISION_TURN_COOLDOWN_RESET;
                    }

                    if vehicle.collision_cooldown_timer <= 0.0 {
                        let damage: f32 = BASE_COLLISION_DAMAGE * abs_vel;
                        debug!("Player {} has collided with {} damage", player.id, damage);

                        let vehicle_destroyed: bool = vehicle_damage_model(
                            vehicle,
                            damage,
                            COLLISION_PIERCING_DAMAGE_PCT,
                            COLLISION_SHIELD_DAMAGE_PCT,
                            COLLISION_ARMOR_DAMAGE_PCT,
                            COLLISION_HEALTH_DAMAGE_PCT,
                        );

                        if vehicle_destroyed {
                            player_destroyed.push(player.id.clone());
                        }

                        if abs_vel > 0.5 {
                            play_bounce_sound(
                                &*sounds,
                                &storage,
                                audio_output.as_deref(),
                            );
                        }

                        vehicle.collision_cooldown_timer = 1.0;
                    }
                }
            }
        }

        for (player, vehicle, transform) in (&mut players, &mut vehicles, &mut transforms).join() {
            for destroyed_id in &player_destroyed {
                if *destroyed_id == player.id {
                    kill_restart_vehicle(vehicle, transform);
                }
            }

            for bounced_id in &player_arena_bounce {
                if *bounced_id == player.id {
                    let vehicle_x = transform.translation().x;
                    let vehicle_y = transform.translation().y;

                    transform.set_translation_x(vehicle_x + vehicle.dx);
                    transform.set_translation_y(vehicle_y + vehicle.dy);
                }
            }
        }
    }
}
