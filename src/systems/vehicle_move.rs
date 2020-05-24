use amethyst::{
    core::{math::Vector3, Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData, World,
        WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::{palette::Srgba, resources::Tint},
    assets::AssetStorage,
    audio::{output::Output, Source},
};

use log::{debug};
use rand::Rng;
use std::f32::consts::PI;
use std::collections::HashMap;

extern crate nalgebra as na;
use na::{Isometry2, Vector2};
use ncollide2d::query::{self, Proximity};
use ncollide2d::shape::{Ball, Cuboid};

use crate::components::{
    check_respawn_vehicle, get_random_weapon_name, kill_restart_vehicle, update_weapon_icon,
    update_weapon_properties, vehicle_damage_model, BotMode, Hitbox, HitboxShape, Player,
    PlayerWeaponIcon, RaceCheckpointType, Vehicle, VehicleState, Weapon, WeaponStoreResource,
};

use crate::entities::{malfunction_sparking, acceleration_spray};

use crate::resources::{GameModeSetup, GameModes, WeaponFireResource};

use crate::rally::{
    ARENA_HEIGHT, ARENA_WIDTH, BASE_COLLISION_DAMAGE, COLLISION_ARMOR_DAMAGE_PCT,
    COLLISION_HEALTH_DAMAGE_PCT, COLLISION_PIERCING_DAMAGE_PCT, COLLISION_SHIELD_DAMAGE_PCT,
    UI_HEIGHT,
};

use crate::audio::{play_bounce_sound, Sounds};

const BOT_COLLISION_TURN_COOLDOWN_RESET: f32 = 0.7;
const BOT_COLLISION_MOVE_COOLDOWN_RESET: f32 = 0.7;

const BOT_ENGAGE_DISTANCE: f32 = 160.0;
const BOT_DISENGAGE_DISTANCE: f32 = 240.0;

const WALL_HIT_BOUNCE_DECEL_PCT: f32 = 0.35;

const ROCKET_SPRAY_COOLDOWN_RESET: f32 = 0.05;


#[derive(SystemDesc, Default)]
pub struct VehicleMoveSystem {
    pub last_spawn_index: u32,
    pub rocket_spray_timer: f32,
}

impl<'s> System<'s> for VehicleMoveSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Hitbox>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Weapon>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
        WriteStorage<'s, Tint>,
        ReadExpect<'s, GameModeSetup>,
        ReadStorage<'s, PlayerWeaponIcon>,
        ReadExpect<'s, WeaponFireResource>,
        ReadExpect<'s, LazyUpdate>,
        ReadExpect<'s, WeaponStoreResource>,
    );

    fn setup(&mut self, _world: &mut World) {
        let mut rng = rand::thread_rng();
        self.last_spawn_index = rng.gen_range(0, 4);
    }

    fn run(
        &mut self,
        (
            entities,
            hitboxes,
            mut players,
            mut transforms,
            mut vehicles,
            mut weapons,
            time,
            input,
            storage,
            sounds,
            audio_output,
            mut tints,
            game_mode_setup,
            player_weapon_icons,
            weapon_fire_resource,
            lazy_update,
            weapon_store_resource,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        let dt = time.delta_seconds();

        self.rocket_spray_timer -= dt;

        let mut weapon_icons_old_map = HashMap::new();

        let mut earned_collision_kills: Vec<usize> = Vec::new();

        //Turn and Accel
        for (player, vehicle, transform, mut weapon) in
            (&mut players, &mut vehicles, &mut transforms, &mut weapons).join()
        {
            if vehicle.state == VehicleState::InRespawn {
                self.last_spawn_index = check_respawn_vehicle(
                    vehicle,
                    transform,
                    dt,
                    game_mode_setup.game_mode.clone(),
                    self.last_spawn_index,
                );

                //if just now respawned and state changed into VehicleState::Active
                if vehicle.state == VehicleState::Active {
                    if game_mode_setup.random_weapon_spawns && !game_mode_setup.keep_picked_up_weapons {
                        let restart_weapon_name = game_mode_setup.starter_weapon.clone();

                        weapon_icons_old_map.insert(player.id, weapon.stats.weapon_type);

                        update_weapon_properties(weapon, restart_weapon_name, &weapon_store_resource);
                        update_weapon_icon(
                            &entities,
                            &mut weapon,
                            &weapon_fire_resource,
                            player.id,
                            &lazy_update,
                        );

                        vehicle.weapon_weight = weapon.stats.weight;
                    }
                }
            }

            
            //Similar comment exists in rally.rs
            //typical vehicle weight = 100 at S:100/A:100/H:100 with normal engine efficiency

            //health makes up the main hull of the vehicle, and contributes 20 base weight + 20 per 100 health
            //shields make up 15 weight
            //armor another 25 weight
            //engine another 20 weight

            //typical weapon weight adds about 10.0
            //  for a total of about 110.0

            
            //a lighter racing vehicle with s:25/A:0/H:100 would weigh:
            //  B:20 + H:20 + S:3.75 + E:20 + W:10 = 73.75,
            //  and therefore would have about 50% quicker acceleration
            //  but could only take about 42% typical damage before blowing up

            //a heavy-weight tank combat vehicle with s:200/A:200/H:150 would weigh:
            //  B:20 + H:30 + S:30 + A:50 + E:20 + W:10 = 160,
            //  and therefore would have about 45% slower acceleration
            //  but would take almost 550 damage, an 83% increase


            //NOTE: lost armor does not contribute to weight, only the current value of armor matters
            let vehicle_weight = (20.0 + vehicle.health.max * 20. / 100.)
                + (vehicle.shield.max * 15. / 100.)
                + (vehicle.armor.value * 25. / 100.)
                + vehicle.engine_weight
                + vehicle.weapon_weight;


            let rotate_accel_rate: f32 = 1.0 * vehicle.engine_force / vehicle_weight;
            let rotate_friction_decel_rate: f32 = 0.98 * vehicle.engine_force / vehicle_weight;

            let thrust_accel_rate: f32 = 0.9 * vehicle.engine_force / vehicle_weight;
            let thrust_decel_rate: f32 = 0.6 * vehicle.engine_force / vehicle_weight;
            let thrust_friction_decel_rate: f32 = 0.3 * vehicle.engine_force / vehicle_weight;

            let wall_hit_non_bounce_decel_pct: f32 = WALL_HIT_BOUNCE_DECEL_PCT;
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

            let vehicle_x = transform.translation().x;
            let vehicle_y = transform.translation().y;

            let vehicle_rotation = transform.rotation();
            let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();

            player.bot_move_cooldown -= dt;

            //Issue Bot commands
            if player.is_bot && vehicle.state == VehicleState::Active {
                if player.bot_mode == BotMode::Running
                    || player.bot_mode == BotMode::TakeTheHill
                    || player.bot_mode == BotMode::Mining
                    || player.bot_mode == BotMode::Repairing
                {
                    if let Some(dist_to_closest_vehicle) = vehicle.dist_to_closest_vehicle {
                        if dist_to_closest_vehicle <= BOT_ENGAGE_DISTANCE
                            && player.bot_move_cooldown < 0.0
                        {
                            //change modes to attack
                            if weapon.stats.attached {
                                //Typically just LaserSword
                                player.bot_mode = BotMode::Swording;
                                debug!("{} Swording", player.id);
                                player.bot_move_cooldown = 5.0;
                            } else if weapon.stats.shot_speed <= 0.0 {
                                //Typically just Mines or Traps
                                player.bot_mode = BotMode::Mining;
                                debug!("{} Mining", player.id);
                            } else {
                                player.bot_mode = BotMode::StopAim;
                                debug!("{} StopAim", player.id);
                                player.bot_move_cooldown = 5.0;
                            }
                        }
                    }

                    if player.bot_mode == BotMode::TakeTheHill {

                    } 
                    else if player.bot_mode == BotMode::Running
                        || player.bot_mode == BotMode::Mining
                        || player.bot_mode == BotMode::Repairing
                    {
                        //continue with Running or Mining mode
                        if player.bot_move_cooldown < 0.0 {
                            //issue new move command

                            if let Some(dist_to_closest_vehicle) = vehicle.dist_to_closest_vehicle {
                                if (vehicle.health.value < vehicle.health.max ||
                                        vehicle.shield.value == 0.0) && 
                                        dist_to_closest_vehicle > BOT_DISENGAGE_DISTANCE {
                                    player.bot_mode = BotMode::Repairing;
                                }
                            }

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
                    let continue_with_attacking_mode;

                    if let Some(dist_to_closest_vehicle) = vehicle.dist_to_closest_vehicle {
                        if dist_to_closest_vehicle > BOT_DISENGAGE_DISTANCE
                            || player.bot_move_cooldown < 0.0
                        {
                            continue_with_attacking_mode = false;

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
                            if dist_to_closest_vehicle > weapon.range_calc {
                                player.bot_mode = BotMode::Chasing;
                                debug!("{} Chasing", player.id);
                            }
                            continue_with_attacking_mode = true;
                        }
                    } else {
                        continue_with_attacking_mode = false;

                        player.bot_move_cooldown = player.bot_move_cooldown_reset;

                        let run_or_chase = rng.gen::<bool>();

                        if run_or_chase {
                            player.bot_mode = BotMode::Running;
                            debug!("{} Running", player.id);
                        } else {
                            player.bot_mode = BotMode::Chasing;
                            debug!("{} Chasing", player.id);
                        }
                    }

                    if continue_with_attacking_mode {
                        //continue with Attacking mode

                        if let Some(attack_angle) = vehicle.angle_to_closest_vehicle {
                            let turn_value = 1.0;

                            //Prepare magnitude of Turning and Acceleration input
                            if player.bot_mode == BotMode::Swording {
                                if weapon.stats.mounted_angle > PI / 2.0
                                    || weapon.stats.mounted_angle < -PI / 2.0
                                {
                                    vehicle_accel = Some(-1.0); //drive backwards sword fighting
                                } else {
                                    vehicle_accel = Some(1.0); //drive forwards sword fighting
                                }
                            } else if player.bot_mode == BotMode::Chasing {
                                vehicle_accel = Some(0.6);
                            }

                            //Solve for Angle and Direction to turn
                            let mut angle_diff =
                                vehicle_angle + weapon.stats.mounted_angle - attack_angle;

                            if angle_diff > PI {
                                angle_diff = -(2.0 * PI - angle_diff);
                            } else if angle_diff < -PI {
                                angle_diff = -(-2.0 * PI - angle_diff);
                            }

                            if angle_diff > 0.001 {
                                vehicle_turn = Some(-turn_value);
                            } else if angle_diff < -0.001 {
                                vehicle_turn = Some(turn_value);
                            } else {
                                vehicle_turn = Some(0.0);
                            }
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

            let yaw_x_comp = -vehicle_angle.sin(); //left is -, right is +
            let yaw_y_comp = vehicle_angle.cos(); //up is +, down is -


            //Apply malfunction for damaged vehicles
            if vehicle.state == VehicleState::Active {
                vehicle.malfunction_cooldown_timer -= dt;

                //if vehicle low on health, or has taken ion damage, or is currently malfunctioning
                if vehicle.health.value <= (0.5 * vehicle.health.max) || 
                        vehicle.ion_malfunction_pct > 0.0 ||
                        vehicle.malfunction > 0.0
                {
                    vehicle.malfunction_cooldown_timer -= dt;

                    if vehicle.malfunction_cooldown_timer < 0.0 {
                        let malfunction_chance = Some(rng.gen_range(0.0, 1.0) as f32).unwrap();

                        //if health is 50: 0-25 never malfunction, 25-75 chance no malfunction, 75-100 chance malfunction
                        //  so 75% no malfunction, 25% malfunction
                        //if health is 25: 0-25 never malfunction, 25-50 chance no malfunction, 50-100 chance malfunction
                        //  so 50% no malfunction, 50% malfunction
                        //if health is 10: 0-25 never malfunction, 25-35 chance no malfunction, 35-100 chance malfunction
                        //  so 35% no malfunction, 65% malfunction

                        let malfunction_occurs: bool;
                        if malfunction_chance - 0.25 > (vehicle.health.value / vehicle.health.max) ||
                                malfunction_chance > (1.-vehicle.ion_malfunction_pct/100.) {
                            malfunction_occurs = true;
                        }
                        else {
                            malfunction_occurs = false;
                        }

                        if malfunction_occurs {
                            vehicle.malfunction = 100.0;

                            let sparks_position = Vector3::new(vehicle_x, vehicle_y, 0.5);

                            malfunction_sparking(
                                &entities,
                                &weapon_fire_resource,
                                sparks_position,
                                &lazy_update,
                            );
                        }
                        else {
                            vehicle.malfunction = 0.0; //no malfunction
                        }
                        
                        vehicle.malfunction_cooldown_timer = 0.5; //reset timer
                        vehicle.ion_malfunction_pct = 0.0; //clear ion malfunctions
                    }
                    //else unchanged, use old malfunction value
                }
                else {
                    vehicle.malfunction_cooldown_timer = -1.0;
                    vehicle.malfunction = 0.0;
                    vehicle.ion_malfunction_pct = 0.0; //clear ion malfunctions
                }
            }

            //Update vehicle velocity from vehicle speed accel input
            if vehicle.state == VehicleState::Active {
                if let Some(move_amount) = vehicle_accel {
                    let scaled_amount: f32 = if vehicle.repair.activated {
                        0.0 as f32
                    } else if vehicle.malfunction > 0.0 {
                        thrust_accel_rate * move_amount * (100.0-vehicle.malfunction) as f32
                    } else if move_amount > 0.0 {
                        thrust_accel_rate * move_amount as f32
                    } else {
                        thrust_decel_rate * move_amount as f32
                    };

                    vehicle.dx += scaled_amount * yaw_x_comp * dt;
                    vehicle.dy += scaled_amount * yaw_y_comp * dt;

                    let position = Vector3::new(
                        vehicle_x - yaw_x_comp*vehicle.height/2.0,
                        vehicle_y - yaw_y_comp*vehicle.height/2.0, 
                        0.5
                    );

                    let is_smoking = (vehicle.health.value / vehicle.health.max) < 4./5.;

                    if scaled_amount >= 0.01 && self.rocket_spray_timer < 0.0 {
                        acceleration_spray(
                            &entities,
                            &weapon_fire_resource,
                            is_smoking,
                            position,
                            vehicle_angle + PI,
                            scaled_amount.abs()*80.0,
                            &lazy_update,
                        );
                    }
                }
            }

            //Apply friction
            //this needs to be applied to vehicle momentum angle, not vehicle_angle angle
            let velocity_angle = vehicle.dy.atan2(vehicle.dx) - (PI / 2.0); //rotate by PI/2 to line up with vehicle_angle angle
            let velocity_x_comp = -velocity_angle.sin(); //left is -, right is +
            let velocity_y_comp = velocity_angle.cos(); //up is +, down is -

            vehicle.dx -= thrust_friction_decel_rate * velocity_x_comp * dt;
            vehicle.dy -= thrust_friction_decel_rate * velocity_y_comp * dt;

            let sq_vel = vehicle.dx.powi(2) + vehicle.dy.powi(2);
            let abs_vel = sq_vel.sqrt();

            if abs_vel > vehicle.max_velocity {
                vehicle.dx *= vehicle.max_velocity / abs_vel;
                vehicle.dy *= vehicle.max_velocity / abs_vel;
            }

            //Transform on vehicle velocity
            if vehicle.dx.abs() > 0.001 {
                transform.prepend_translation_x(vehicle.dx);
            }

            if vehicle.dy.abs() > 0.001 {
                transform.prepend_translation_y(vehicle.dy);
            }

            //Apply vehicle rotation from turn input
            if vehicle.state == VehicleState::Active {
                if let Some(turn_amount) = vehicle_turn {
                    let scaled_amount: f32 = if vehicle.repair.activated == true {
                        0.0 as f32
                    } else if vehicle.malfunction > 0.0 {
                        rotate_accel_rate * turn_amount * (100.0-vehicle.malfunction) as f32
                    } else {
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
            }

            //Wall-collision logic
            let veh_rect_width = vehicle.height * 0.5 * yaw_x_comp.abs()
                + vehicle.width * 0.5 * (1.0 - yaw_x_comp.abs());
            let veh_rect_height = vehicle.height * 0.5 * yaw_y_comp.abs()
                + vehicle.width * 0.5 * (1.0 - yaw_y_comp.abs());

            let mut x_collision = false;
            let mut y_collision = false;

            if vehicle_x > (ARENA_WIDTH - veh_rect_width) {
                //hit the right wall
                transform.set_translation_x(ARENA_WIDTH - veh_rect_width);
                x_collision = true;
            } else if vehicle_x < (veh_rect_width) {
                //hit the left wall
                transform.set_translation_x(veh_rect_width);
                x_collision = true;
            }

            if vehicle_y > (ARENA_HEIGHT - veh_rect_height) {
                //hit the top wall
                transform.set_translation_y(ARENA_HEIGHT - veh_rect_height);
                y_collision = true;
            } else if vehicle_y < (UI_HEIGHT + veh_rect_height) {
                //hit the bottom wall
                transform.set_translation_y(UI_HEIGHT + veh_rect_height);
                y_collision = true;
            }

            if x_collision {
                vehicle.dx *= wall_hit_bounce_decel_pct * velocity_x_comp.abs();
                vehicle.dy *= wall_hit_non_bounce_decel_pct * velocity_y_comp.abs();

                if vehicle.state == VehicleState::Active {
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
                            player.deaths += 2; //self-destruct counts for 2

                            if player.last_hit_timer <= game_mode_setup.last_hit_threshold {
                                if let Some(last_hit_by_id) = player.last_hit_by_id {
                                    earned_collision_kills.push(last_hit_by_id);
                                }
                            }

                            kill_restart_vehicle(
                                player,
                                vehicle,
                                transform,
                                game_mode_setup.stock_lives,
                            );
                        }

                        if abs_vel > 0.5 {
                            play_bounce_sound(&*sounds, &storage, audio_output.as_deref());
                        }
                        vehicle.collision_cooldown_timer = 1.0;
                    }
                }
            }
            if y_collision {
                vehicle.dx *= wall_hit_non_bounce_decel_pct * velocity_x_comp.abs();
                vehicle.dy *= wall_hit_bounce_decel_pct * velocity_y_comp.abs();

                if vehicle.state == VehicleState::Active {
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
                            player.deaths += 2; //self-destruct counts for 2

                            if player.last_hit_timer <= game_mode_setup.last_hit_threshold {
                                if let Some(last_hit_by_id) = player.last_hit_by_id {
                                    earned_collision_kills.push(last_hit_by_id);
                                }
                            }

                            kill_restart_vehicle(
                                player,
                                vehicle,
                                transform,
                                game_mode_setup.stock_lives,
                            );
                        }

                        if abs_vel > 0.5 {
                            play_bounce_sound(&*sounds, &storage, audio_output.as_deref());
                        }
                        vehicle.collision_cooldown_timer = 1.0;
                    }
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

        //hitbox collision logic
        let mut player_destroyed: Vec<usize> = Vec::new();

        let mut player_arena_bounce_map = HashMap::new();

        let mut players_on_hill: Vec<usize> = Vec::new();
        let mut color_for_hill: Vec<(f32, f32, f32)> = Vec::new();

        for (player, vehicle, mut weapon, transform) in
            (&mut players, &mut vehicles, &mut weapons, &transforms).join()
        {
            let wall_hit_non_bounce_decel_pct: f32 = WALL_HIT_BOUNCE_DECEL_PCT;
            let wall_hit_bounce_decel_pct: f32 = -wall_hit_non_bounce_decel_pct;

            let vehicle_x = transform.translation().x;
            let vehicle_y = transform.translation().y;

            let vehicle_rotation = transform.rotation();
            let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();

            let collision_margin = 5.0;

            let vehicle_collider_shape = Cuboid::new(Vector2::new(vehicle.width/2.0, vehicle.height/2.0));
            let vehicle_collider_pos = Isometry2::new(Vector2::new(vehicle_x, vehicle_y), vehicle_angle);

            for (hitbox_entity, hitbox, hitbox_transform) in
                (&*entities, &hitboxes, &transforms).join()
            {
                let hitbox_x = hitbox_transform.translation().x;
                let hitbox_y = hitbox_transform.translation().y;

                let hit;
                let mut contact_data = None;

                if hitbox.shape == HitboxShape::Circle {
                    let hitbox_collider_shape = Ball::new(hitbox.width / 2.0);
                    let hitbox_collider_pos = Isometry2::new(Vector2::new(hitbox_x, hitbox_y), 0.0);

                    let collision = query::proximity(
                        &vehicle_collider_pos, &vehicle_collider_shape,
                        &hitbox_collider_pos, &hitbox_collider_shape,
                        collision_margin,
                    );

                    if collision == Proximity::Intersecting {
                        hit = true;

                        contact_data = query::contact(
                            &vehicle_collider_pos, &vehicle_collider_shape,
                            &hitbox_collider_pos, &hitbox_collider_shape,
                            0.0);
                    }
                    else if collision == Proximity::WithinMargin {
                        hit = false;
                    }
                    else {
                        hit = false;
                    }
                } else if hitbox.shape == HitboxShape::Rectangle {
                    let hitbox_collider_shape = Cuboid::new(Vector2::new(hitbox.width/2.0, hitbox.height/2.0));
                    let hitbox_collider_pos = Isometry2::new(Vector2::new(hitbox_x, hitbox_y), 0.0);

                    let collision = query::proximity(
                        &vehicle_collider_pos, &vehicle_collider_shape,
                        &hitbox_collider_pos, &hitbox_collider_shape,
                        collision_margin,
                    );

                    if collision == Proximity::Intersecting {
                        hit = true;

                        contact_data = query::contact(
                            &vehicle_collider_pos, &vehicle_collider_shape,
                            &hitbox_collider_pos, &hitbox_collider_shape,
                            0.0);
                    }
                    else if collision == Proximity::WithinMargin {
                        hit = false;
                    }
                    else {
                        hit = false;
                    }
                } else if hitbox.shape == HitboxShape::InnerQuarterCircle {
                    let hitbox_collider_shape = Cuboid::new(Vector2::new(hitbox.width/2.0, hitbox.height/2.0));
                    let hitbox_collider_pos = Isometry2::new(Vector2::new(hitbox_x, hitbox_y), 0.0);

                    let collision = query::proximity(
                        &vehicle_collider_pos, &vehicle_collider_shape,
                        &hitbox_collider_pos, &hitbox_collider_shape,
                        collision_margin,
                    );

                    if collision == Proximity::Intersecting {
                        hit = true;

                        contact_data = query::contact(
                            &vehicle_collider_pos, &vehicle_collider_shape,
                            &hitbox_collider_pos, &hitbox_collider_shape,
                            0.0);
                    }
                    else if collision == Proximity::WithinMargin {
                        hit = false;
                    }
                    else {
                        hit = false;
                    }
                } else if hitbox.shape == HitboxShape::OuterQuarterCircle {
                    let hitbox_collider_shape = Cuboid::new(Vector2::new(hitbox.width/2.0, hitbox.height/2.0));
                    let hitbox_collider_pos = Isometry2::new(Vector2::new(hitbox_x, hitbox_y), 0.0);

                    let collision = query::proximity(
                        &vehicle_collider_pos, &vehicle_collider_shape,
                        &hitbox_collider_pos, &hitbox_collider_shape,
                        collision_margin,
                    );

                    if collision == Proximity::Intersecting {
                        hit = true;

                        contact_data = query::contact(
                            &vehicle_collider_pos, &vehicle_collider_shape,
                            &hitbox_collider_pos, &hitbox_collider_shape,
                            0.0);
                    }
                    else if collision == Proximity::WithinMargin {
                        hit = false;
                    }
                    else {
                        hit = false;
                    }
                } else {
                    hit = false;
                }

                if hit {
                    if hitbox.is_wall {
                        //let contact_depth = contact_data.unwrap().depth;
                        let contact_pt = contact_data.unwrap().world2;

                        let sq_vel = vehicle.dx.powi(2) + vehicle.dy.powi(2);
                        let abs_vel = sq_vel.sqrt();

                        vehicle.dx *= wall_hit_bounce_decel_pct;
                        vehicle.dy *= wall_hit_bounce_decel_pct;

                        player_arena_bounce_map.insert(player.id.clone(), contact_pt);

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

                                player.deaths += 2; //self-destruct counts for 2

                                if player.last_hit_timer <= game_mode_setup.last_hit_threshold {
                                    if let Some(last_hit_by_id) = player.last_hit_by_id {
                                        earned_collision_kills.push(last_hit_by_id);
                                    }
                                }
                            }

                            if abs_vel > 0.5 {
                                play_bounce_sound(&*sounds, &storage, audio_output.as_deref());
                            }

                            vehicle.collision_cooldown_timer = 1.0;
                        }
                    } else if vehicle.state == VehicleState::Active {
                        //Non-collision related actions can only occur on Active vehicles
                        if hitbox.is_weapon_box {
                            let _ = entities.delete(hitbox_entity);

                            let new_weapon_name = get_random_weapon_name(&game_mode_setup);

                            weapon_icons_old_map.insert(player.id, weapon.stats.weapon_type);

                            update_weapon_properties(
                                weapon,
                                new_weapon_name,
                                &weapon_store_resource,
                            );
                            update_weapon_icon(
                                &entities,
                                &mut weapon,
                                &weapon_fire_resource,
                                player.id,
                                &lazy_update,
                            );

                            vehicle.weapon_weight = weapon.stats.weight;
                        } else if hitbox.is_hill {
                            players_on_hill.push(player.id.clone());

                            let (r, g, b) = match player.id.clone() {
                                0 => (1.0, 0.3, 0.3),
                                1 => (0.3, 0.3, 1.0),
                                2 => (0.3, 1.0, 0.3),
                                3 => (1.0, 0.8, 0.3),
                                _ => (1.0, 1.0, 1.0),
                            };

                            color_for_hill.push((r, g, b));
                        } else if (hitbox.checkpoint == RaceCheckpointType::Checkpoint)
                                && (hitbox.checkpoint_id == player.checkpoint_completed + 1)
                        {
                            player.checkpoint_completed = hitbox.checkpoint_id;
                            debug!("checkpoints:{}", player.checkpoint_completed);
                        } 
                        else if hitbox.checkpoint == RaceCheckpointType::Lap {
                            if player.checkpoint_completed == game_mode_setup.checkpoint_count {
                                player.laps_completed += 1;
                            }
                            player.checkpoint_completed = 0;
                            debug!("checkpoints:{}", player.checkpoint_completed);
                            debug!("laps:{}", player.laps_completed);
                        }
                    }
                }
            }
        }

        for (player, vehicle, transform) in (&mut players, &mut vehicles, &mut transforms).join() {
            if players_on_hill.len() == 1 && players_on_hill.contains(&player.id) {
                player.objective_points += dt;
            }

            if player_destroyed.contains(&player.id) {
                kill_restart_vehicle(player, vehicle, transform, game_mode_setup.stock_lives);
            }

            if game_mode_setup.game_mode != GameModes::ClassicGunGame {
                player.kills += earned_collision_kills
                    .iter()
                    .filter(|&n| *n == player.id)
                    .count() as i32;
            }

            let player_bounce = player_arena_bounce_map.get(&player.id);

            if let Some(player_bounce) = player_bounce {
                let contact_x = player_bounce.x;
                let contact_y = player_bounce.y;

                let vehicle_x = transform.translation().x;
                let vehicle_y = transform.translation().y;

                transform.set_translation_x(vehicle_x - (contact_x - vehicle_x)/10. + vehicle.dx);
                transform.set_translation_y(vehicle_y - (contact_y - vehicle_y)/10. + vehicle.dy);
            }
        }

        for (entity, _hitbox) in (&*entities, &hitboxes).join() {
            if let Some(tint) = tints.get_mut(entity) {
                if players_on_hill.len() == 1 {
                    *tint = Tint(Srgba::new(
                        color_for_hill[0].0,
                        color_for_hill[0].1,
                        color_for_hill[0].2,
                        1.0,
                    ));
                } else {
                    *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));
                }
            }
        }

        for (entity, player_icon) in (&*entities, &player_weapon_icons).join() {
            let weapon_icons_old = weapon_icons_old_map.get(&player_icon.id);

            if let Some(weapon_icons_old) = weapon_icons_old {
                let weapon_type = weapon_icons_old;
                if *weapon_type == player_icon.weapon_type {
                    let _ = entities.delete(entity);
                }
            }
        }

        if self.rocket_spray_timer < 0.0 {
            self.rocket_spray_timer = ROCKET_SPRAY_COOLDOWN_RESET;
        }
    }
}