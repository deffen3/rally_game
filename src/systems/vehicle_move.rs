use amethyst::core::{Time, Transform};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{
    Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData, World,
    WriteStorage,
};
use amethyst::input::{InputHandler, StringBindings};
use amethyst::renderer::{palette::Srgba, resources::Tint};
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
};

use log::debug;
use rand::Rng;
use std::f32::consts::PI;

use std::collections::HashMap;

use crate::components::{
    check_respawn_vehicle, get_random_weapon_name, kill_restart_vehicle, update_weapon_icon,
    update_weapon_properties, BotMode, Hitbox, HitboxShape, Player, PlayerWeaponIcon,
    RaceCheckpointType, Vehicle, VehicleState, Weapon,
};
use crate::resources::{GameModeSetup, WeaponFireResource};

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

#[derive(SystemDesc, Default)]
pub struct VehicleMoveSystem {
    pub last_spawn_index: u32,
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
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        let dt = time.delta_seconds();

        let mut weapon_icons_old_map = HashMap::new();

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

                if game_mode_setup.random_weapon_spawns && !game_mode_setup.keep_picked_up_weapons {
                    let restart_weapon_name = game_mode_setup.starter_weapon.clone();

                    weapon_icons_old_map.insert(player.id, weapon.stats.weapon_type);

                    update_weapon_properties(weapon, restart_weapon_name);
                    update_weapon_icon(
                        &entities,
                        &mut weapon,
                        &weapon_fire_resource,
                        player.id,
                        &lazy_update,
                    );

                    vehicle.weapon_weight = weapon.stats.weight;
                }
            } else if vehicle.state == VehicleState::Active {
                //lost armor does not contribute to weight
                let vehicle_weight = (30.0 + vehicle.health.max * 10. / 100.)
                    + (vehicle.armor.value * 25. / 100.)
                    + (vehicle.shield.max * 15. / 100.)
                    + vehicle.engine_weight
                    + vehicle.weapon_weight;

                let rotate_accel_rate: f32 = 1.0 * vehicle.engine_force / vehicle_weight;
                let rotate_friction_decel_rate: f32 = 0.98 * vehicle.engine_force / vehicle_weight;

                let thrust_accel_rate: f32 = 0.9 * vehicle.engine_force / vehicle_weight;
                let thrust_decel_rate: f32 = 0.6 * vehicle.engine_force / vehicle_weight;
                let thrust_friction_decel_rate: f32 = 0.3 * vehicle.engine_force / vehicle_weight;

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
                    if player.bot_mode == BotMode::Running
                        || player.bot_mode == BotMode::TakeTheHill
                        || player.bot_mode == BotMode::Mining
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
                        } else if player.bot_mode == BotMode::Running
                            || player.bot_mode == BotMode::Mining
                        {
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
                if let Some(turn_amount) = vehicle_turn {
                    let scaled_amount: f32 = if vehicle.repair.activated == true {
                        0.0 as f32
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

                //Wall-collision logic
                let vehicle_x = transform.translation().x;
                let vehicle_y = transform.translation().y;

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

        //hitbox collision logic
        let mut player_destroyed: Vec<usize> = Vec::new();
        let mut player_arena_bounce: Vec<usize> = Vec::new();

        let mut players_on_hill: Vec<usize> = Vec::new();
        let mut color_for_hill: Vec<(f32, f32, f32)> = Vec::new();

        for (player, vehicle, mut weapon, transform) in
            (&mut players, &mut vehicles, &mut weapons, &transforms).join()
        {
            if vehicle.state == VehicleState::Active {
                let wall_hit_non_bounce_decel_pct: f32 = 0.65;
                let wall_hit_bounce_decel_pct: f32 = -wall_hit_non_bounce_decel_pct;

                let vehicle_x = transform.translation().x;
                let vehicle_y = transform.translation().y;

                let vehicle_rotation = transform.rotation();
                let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();

                let yaw_x_comp = -vehicle_angle.sin(); //left is -, right is +
                let yaw_y_comp = vehicle_angle.cos(); //up is +, down is -

                let veh_rect_width = vehicle.height * 0.5 * yaw_x_comp.abs()
                    + vehicle.width * 0.5 * (1.0 - yaw_x_comp.abs());

                let veh_rect_height = vehicle.height * 0.5 * yaw_y_comp.abs()
                    + vehicle.width * 0.5 * (1.0 - yaw_y_comp.abs());

                let mut checkpoint_id: i32 = 0;
                let mut checkpoint_stages: [bool; 2] = [false; 2];
                let mut lap_stages: [bool; 2] = [false; 2];

                for (hitbox_entity, hitbox, hitbox_transform) in
                    (&*entities, &hitboxes, &transforms).join()
                {
                    let hitbox_x = hitbox_transform.translation().x;
                    let hitbox_y = hitbox_transform.translation().y;

                    let hit;
                    if hitbox.shape == HitboxShape::Circle {
                        if (vehicle_x - hitbox_x).powi(2) + (vehicle_y - hitbox_y).powi(2)
                            < (hitbox.width / 2.0 + vehicle.width / 2.0).powi(2)
                        {
                            hit = true;
                        } else {
                            hit = false;
                        }
                    } else if hitbox.shape == HitboxShape::Rectangle {
                        let mut interferences = 0;

                        let left_hitbox_wall =
                            vehicle_x + veh_rect_width / 2.0 > hitbox_x - hitbox.width / 2.0;
                        let right_hitbox_wall =
                            vehicle_x - veh_rect_width / 2.0 < hitbox_x + hitbox.width / 2.0;

                        let top_hitbox_wall =
                            vehicle_y - veh_rect_height / 2.0 < hitbox_y + hitbox.height / 2.0;
                        let bottom_hitbox_wall =
                            vehicle_y + veh_rect_height / 2.0 > hitbox_y - hitbox.height / 2.0;

                        if left_hitbox_wall {
                            interferences += 1;
                        }
                        if right_hitbox_wall {
                            interferences += 1;
                        }
                        if top_hitbox_wall {
                            interferences += 1;
                        }
                        if bottom_hitbox_wall {
                            interferences += 1;
                        }

                        if interferences >= 4 {
                            hit = true;
                        } else {
                            hit = false;
                        }
                    } else {
                        hit = false;
                    }

                    if hit {
                        if hitbox.is_wall {
                            let sq_vel = vehicle.dx.powi(2) + vehicle.dy.powi(2);
                            let abs_vel = sq_vel.sqrt();

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
                                    play_bounce_sound(&*sounds, &storage, audio_output.as_deref());
                                }

                                vehicle.collision_cooldown_timer = 1.0;
                            }
                        } else if hitbox.is_weapon_box {
                            let _ = entities.delete(hitbox_entity);

                            let new_weapon_name = get_random_weapon_name();

                            weapon_icons_old_map.insert(player.id, weapon.stats.weapon_type);

                            update_weapon_properties(weapon, new_weapon_name);
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
                        } else if (hitbox.checkpoint == RaceCheckpointType::CheckpointStart)
                            && (hitbox.checkpoint_id == player.checkpoint_completed + 1)
                        {
                            checkpoint_stages[0] = true;
                            checkpoint_id = hitbox.checkpoint_id;
                        } else if hitbox.checkpoint == RaceCheckpointType::CheckpointFinish
                            && (hitbox.checkpoint_id == player.checkpoint_completed + 1)
                        {
                            checkpoint_stages[1] = true;
                            checkpoint_id = hitbox.checkpoint_id;
                        } else if hitbox.checkpoint == RaceCheckpointType::LapStart {
                            lap_stages[0] = true;
                        } else if hitbox.checkpoint == RaceCheckpointType::LapFinish {
                            lap_stages[1] = true;
                        }
                    }
                }

                // println!("C: {} {}",checkpoint_stages[0], checkpoint_stages[1]);
                // println!("L: {} {}",lap_stages[0], lap_stages[1]);

                if checkpoint_stages[0] == true && checkpoint_stages[1] == false {
                    player.hit_checkpoint_start = true;
                } else if checkpoint_stages[0] == true
                    && checkpoint_stages[1] == true
                    && player.hit_checkpoint_start
                {
                    player.hit_checkpoint_middle = true;
                } else if checkpoint_stages[0] == false
                    && checkpoint_stages[1] == true
                    && player.hit_checkpoint_middle
                {
                    player.checkpoint_completed = checkpoint_id;
                    player.hit_checkpoint_start = false;
                    player.hit_checkpoint_middle = false;
                } else {
                    player.hit_checkpoint_start = false;
                    player.hit_checkpoint_middle = false;
                }

                if lap_stages[0] == true && lap_stages[1] == false {
                    player.hit_lap_start = true;
                } else if lap_stages[0] == true && lap_stages[1] == true && player.hit_lap_start {
                    player.hit_lap_middle = true;
                } else if lap_stages[0] == false && lap_stages[1] == true {
                    if player.hit_lap_middle
                        && (player.checkpoint_completed == game_mode_setup.checkpoint_count)
                    {
                        player.laps_completed += 1;
                        player.hit_lap_start = false;
                        player.hit_lap_middle = false;
                    } else {
                        //wrong-way detection
                        player.hit_lap_start = false;
                        player.hit_lap_middle = false;
                        player.checkpoint_completed = 0;
                    }
                } else {
                    player.hit_lap_start = false;
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

            if player_arena_bounce.contains(&player.id) {
                let vehicle_x = transform.translation().x;
                let vehicle_y = transform.translation().y;

                transform.set_translation_x(vehicle_x + vehicle.dx);
                transform.set_translation_y(vehicle_y + vehicle.dy);
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
    }
}
