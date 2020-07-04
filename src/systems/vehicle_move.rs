use amethyst::{
    core::{math::Vector3, Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, Write, System, SystemData, World,
        WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::{debug_drawing::{DebugLines}, palette::Srgba, resources::Tint},
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
    check_respawn_vehicle, get_random_weapon_name, get_random_weapon_name_build_chance, kill_restart_vehicle,
    update_weapon_properties, vehicle_damage_model, BotMode, ArenaElement, HitboxShape, Player,
    PlayerWeaponIcon, RaceCheckpointType, Vehicle, VehicleState, WeaponArray, WeaponStoreResource,
    determine_vehicle_weight, VehicleMovementType, DurationDamage, 
    ArenaStoreResource, ArenaNames, ArenaProperties, ObstacleType,
};

use crate::entities::{malfunction_sparking, acceleration_spray};

use crate::resources::{GameModeSetup, GameModes, GameWeaponSetup, WeaponFireResource, GameWeaponMode};

use crate::rally::{
    BASE_COLLISION_DAMAGE, COLLISION_ARMOR_DAMAGE_PCT,
    COLLISION_HEALTH_DAMAGE_PCT, COLLISION_PIERCING_DAMAGE_PCT, COLLISION_SHIELD_DAMAGE_PCT,
    DEBUG_LINES,
};

use crate::audio::{play_bounce_sound, Sounds};

const BOT_COLLISION_TURN_COOLDOWN_RESET: f32 = 0.3;
const BOT_COLLISION_MOVE_COOLDOWN_RESET: f32 = 0.3;

const BOT_ENGAGE_DISTANCE: f32 = 160.0;
const BOT_DISENGAGE_DISTANCE: f32 = 240.0;

const BOT_NO_HIT_MOVE_COOLDOWN: f32 = 2.0;

const WALL_HIT_BOUNCE_DECEL_PCT: f32 = 0.35;

const ROCKET_SPRAY_COOLDOWN_RESET: f32 = 0.05;

const PRIMARY_WEAPON_INDEX: usize = 0;
const SECONDARY_WEAPON_INDEX: usize = 1;


#[derive(SystemDesc, Default)]
pub struct VehicleMoveSystem {
    pub last_spawn_index: u32,
    pub rocket_spray_timer: f32,
    pub arena_properties: ArenaProperties,
}

impl<'s> System<'s> for VehicleMoveSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, ArenaElement>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, WeaponArray>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
        WriteStorage<'s, Tint>,
        ReadExpect<'s, GameModeSetup>,
        ReadExpect<'s, GameWeaponSetup>,
        ReadStorage<'s, PlayerWeaponIcon>,
        ReadExpect<'s, WeaponFireResource>,
        ReadExpect<'s, LazyUpdate>,
        ReadExpect<'s, WeaponStoreResource>,
        Write<'s, DebugLines>,
    );

    fn setup(&mut self, world: &mut World) {
        let mut rng = rand::thread_rng();
        self.last_spawn_index = rng.gen_range(0, 4);


        let arena_name;
        {
            let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();
    
            if let Some(game_mode_setup) = fetched_game_mode_setup {
                arena_name = game_mode_setup.arena_name.clone();
            } else {
                arena_name = ArenaNames::OpenEmptyMap;
            }
        }
    
        {        
            let fetched_arena_store = world.try_fetch::<ArenaStoreResource>();
    
            if let Some(arena_store) = fetched_arena_store {
                self.arena_properties = match arena_store.properties.get(&arena_name) {
                    Some(arena_props_get) => (*arena_props_get).clone(),
                    _ => ArenaProperties::default(),
                };
            }
            else {
                self.arena_properties = ArenaProperties::default();
            }
        }
    }

    fn run(
        &mut self,
        (
            entities,
            arena_elements,
            mut players,
            mut transforms,
            mut vehicles,
            mut weapon_arrays,
            time,
            input,
            storage,
            sounds,
            audio_output,
            mut tints,
            game_mode_setup,
            game_weapon_setup,
            player_weapon_icons,
            weapon_fire_resource,
            lazy_update,
            weapon_store_resource,
            mut debug_lines_resource,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        let dt = time.delta_seconds();

        self.rocket_spray_timer -= dt;

        let mut weapon_icons_old_map = HashMap::new();

        let mut earned_collision_kills: Vec<usize> = Vec::new();

        //Turn and Accel
        for (player, vehicle, transform, mut weapon_array) in
            (&mut players, &mut vehicles, &mut transforms, &mut weapon_arrays).join()
        {
            if vehicle.state == VehicleState::InRespawn {
                self.last_spawn_index = check_respawn_vehicle(
                    vehicle,
                    transform,
                    dt,
                    game_mode_setup.game_mode.clone(),
                    self.last_spawn_index,
                    &self.arena_properties,
                );

                //if just now respawned and state changed into VehicleState::Active
                if vehicle.state == VehicleState::Active 
                {
                    if game_weapon_setup.new_ammo_on_respawn {
                        for weapon_install in weapon_array.installed.iter_mut() {
                            if !weapon_install.ammo.is_none() {
                                let mut weapon = &mut weapon_install.weapon;
                                weapon.ammo = Some(weapon_install.ammo.unwrap());
                            }
                        }
                    }

                    
                    if !game_weapon_setup.keep_picked_up_weapons &&
                        (game_weapon_setup.mode == GameWeaponMode::StarterAndPickup ||
                        game_weapon_setup.mode == GameWeaponMode::CustomStarterAndPickup)
                    {
                        //Remove weapons picked up in previous life
                        if weapon_array.installed.len() >= 2
                        {
                            let secondary_weapon = &weapon_array.installed[SECONDARY_WEAPON_INDEX].weapon;

                            vehicle.weapon_weight -= secondary_weapon.stats.weight;
                            
                            weapon_icons_old_map.insert(
                                player.id,
                                (SECONDARY_WEAPON_INDEX, secondary_weapon.stats.weapon_fire_type.clone()));

                            update_weapon_properties(
                                &mut weapon_array,
                                SECONDARY_WEAPON_INDEX,
                                1,
                                None,
                                None,
                                &weapon_store_resource,
                                &entities,
                                &weapon_fire_resource,
                                player.id,
                                &lazy_update,
                            );
                        } //else, hadn't picked up a weapon spawn box yet
                    }
                }
            }
            
            let vehicle_weight = determine_vehicle_weight(vehicle);


            let rotate_accel_rate: f32 = 120.0 * vehicle.engine_force / vehicle_weight;
            let rotate_friction_decel_rate: f32 = 75.0 * vehicle.engine_force / vehicle_weight;

            let thrust_accel_rate: f32 = 90.0 * vehicle.engine_force / vehicle_weight;
            let thrust_decel_rate: f32 = 60.0 * vehicle.engine_force / vehicle_weight;
            let thrust_strafe_accel_rate: f32 = 60.0 * vehicle.engine_force / vehicle_weight;
            let thrust_friction_decel_rate: f32 = 30.0 * vehicle.engine_force / vehicle_weight;

            let tire_longitudinal_friction_decel_rate: f32 = 30.0;
            let tire_lateral_friction_decel_rate: f32 = 15.0;

            let tank_track_longitudinal_friction_decel_rate: f32 = 80.0;
            let tank_track_lateral_friction_decel_rate: f32 = 200.0;

            let wall_hit_non_bounce_decel_pct: f32 = WALL_HIT_BOUNCE_DECEL_PCT;
            let wall_hit_bounce_decel_pct: f32 = -wall_hit_non_bounce_decel_pct;

            //let vehicle_accel = input.axis_value(&AxisBinding::VehicleAccel(player.id));
            //let vehicle_turn = input.axis_value(&AxisBinding::VehicleTurn(player.id));

            let (mut vehicle_accel, mut vehicle_turn, mut vehicle_strafe) = match player.id {
                0 => (input.axis_value("p1_accel"), input.axis_value("p1_turn"), input.axis_value("p1_strafe")),
                1 => (input.axis_value("p2_accel"), input.axis_value("p2_turn"), input.axis_value("p2_strafe")),
                2 => (input.axis_value("p3_accel"), input.axis_value("p3_turn"), input.axis_value("p3_strafe")),
                3 => (input.axis_value("p4_accel"), input.axis_value("p4_turn"), input.axis_value("p4_strafe")),
                _ => (None, None, None),
            };

            let vehicle_x = transform.translation().x;
            let vehicle_y = transform.translation().y;

            let vehicle_rotation = transform.rotation();
            let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();
            
            

            //Issue Bot commands
            if player.is_bot {
                player.bot_move_cooldown -= dt;
                player.path_cooldown -= dt;

                if vehicle.state == VehicleState::Active {
                    if player.bot_mode == BotMode::Sleep {
                        //Wakeup-logic needed here
                        //In most game modes the bot should immediately wake up
                        if game_mode_setup.game_mode == GameModes::Race {
                            player.bot_mode = BotMode::Racing;
                            debug!("{} Racing", player.id);
                        }
                        else {
                            player.bot_mode = BotMode::RunRandom;
                            debug!("{} Run Random", player.id);
                        }
                    }
                    else if player.bot_mode == BotMode::Racing {
                        //Determine which point to race to

                        let next_checkpoint;
                        if player.checkpoint_completed as usize >= (self.arena_properties.race_checkpoints.len() - 1) {
                            next_checkpoint = self.arena_properties.race_checkpoints[0]; //return to finish line
                        }
                        else { //go to next checkpoint
                            next_checkpoint = self.arena_properties.race_checkpoints[(player.checkpoint_completed + 1) as usize];
                        }
                        

                        player.path_target = Some((next_checkpoint.x, next_checkpoint.y, 0.0));


                        if let Some(path_plan) = player.path_plan.clone() {
                            if path_plan.len() >= 2 {
                                let first_target = path_plan[1];
                                let path_x = first_target.0;
                                let path_y = first_target.1;

                                let x_diff = vehicle_x - path_x;
                                let y_diff = vehicle_y - path_y;

                                let dist = (x_diff.powi(2) + y_diff.powi(2)).sqrt();

                                let sq_vel = vehicle.dx.powi(2) + vehicle.dy.powi(2);
                                let abs_vel = sq_vel.sqrt(); //Why does velocity not seem accurate without this weird multiplier???

                                let approx_t_to_arrival = dist / abs_vel;
                                let approx_t_to_rest = abs_vel / thrust_friction_decel_rate;

                                let target_angle = y_diff.atan2(x_diff) + (PI / 2.0); //rotate by PI/2 to line up with 0deg is pointed towards top

                                let mut angle_diff = vehicle_angle - target_angle;

                                if angle_diff > PI {
                                    angle_diff = -(2.0 * PI - angle_diff);
                                } else if angle_diff < -PI {
                                    angle_diff = -(-2.0 * PI - angle_diff);
                                }

                                let turn_value = 0.2;

                                if angle_diff > 0.001 {
                                    vehicle_turn = Some(-turn_value);
                                } else if angle_diff < -0.001 {
                                    vehicle_turn = Some(turn_value);
                                } else {
                                    vehicle_turn = Some(0.0);
                                }

                                if angle_diff.abs() < 0.2 {
                                    if approx_t_to_arrival < approx_t_to_rest {
                                        vehicle_accel = Some(0.5);
                                    }
                                    else {
                                        vehicle_accel = Some(0.8);
                                    }
                                }
                            }
                            else {
                                vehicle_accel = Some(0.0);
                                vehicle_turn = Some(0.0);
                            }                                
                        }
                        player.last_accel_input = vehicle_accel;
                        player.last_turn_input = vehicle_turn;

                        if player.bot_move_cooldown < 0.0 {
                            player.bot_move_cooldown = player.bot_move_cooldown_reset;
                        }
                    }
                    else if player.bot_mode == BotMode::RunTo
                        || player.bot_mode == BotMode::RunRandom
                        || player.bot_mode == BotMode::RunBlind
                        || player.bot_mode == BotMode::TakeTheHill
                        || player.bot_mode == BotMode::Mining
                        || player.bot_mode == BotMode::Repairing
                    {
                        if game_mode_setup.game_mode == GameModes::KingOfTheHill && player.on_hill == false {
                            player.bot_mode = BotMode::TakeTheHill;
                            debug!("{} TakeTheHill", player.id);
                        }
                        else {
                            //check to change mode
                            if let Some(dist_to_closest_vehicle) = vehicle.dist_to_closest_vehicle {
                                if dist_to_closest_vehicle <= BOT_ENGAGE_DISTANCE
                                    && player.bot_move_cooldown < 0.0
                                {
                                    if weapon_array.installed.len() > 0 {
                                        let primary_weapon = &weapon_array.installed[PRIMARY_WEAPON_INDEX].weapon;
                                        //change modes to attack
                                        if primary_weapon.stats.fire_stats.attached {
                                            //Typically just LaserSword
                                            player.bot_mode = BotMode::Swording;
                                            debug!("{} Swording", player.id);
                                            player.bot_move_cooldown = 5.0;
                                            player.last_made_hit_timer = 0.0;
                                        } else if primary_weapon.stats.fire_stats.shot_speed <= 0.0 {
                                            //Typically just Mines or Traps
                                            player.bot_mode = BotMode::Mining;
                                            debug!("{} Mining", player.id);
                                        } else {
                                            player.bot_mode = BotMode::StopAim;
                                            debug!("{} StopAim", player.id);
                                            player.bot_move_cooldown = 1.0;
                                            player.last_made_hit_timer = 0.0;
                                        }
                                    }
                                }
                            }
                        }

                        if player.bot_mode == BotMode::RunTo
                            || player.bot_mode == BotMode::RunRandom
                            || player.bot_mode == BotMode::RunBlind
                            || player.bot_mode == BotMode::TakeTheHill
                            || player.bot_mode == BotMode::Mining
                            || player.bot_mode == BotMode::Repairing
                        {
                            if player.bot_mode == BotMode::TakeTheHill {
                                if player.on_hill == false {
                                    player.path_target = Some((self.arena_properties.width/2.0, (self.arena_properties.height)/2.0 , 0.0));
                                }
                                else {
                                    player.path_target = None;
                                }   
                            }
                            else if player.bot_mode == BotMode::RunTo
                                    || player.bot_mode == BotMode::RunRandom
                                    || player.bot_mode == BotMode::Mining
                                    || player.bot_mode == BotMode::Repairing
                            {
                                if player.path_target.is_none() || player.path_cooldown <= 0.0 {
                                    let random_x = rng.gen_range(0.0, self.arena_properties.width) as f32;
                                    let random_y = rng.gen_range(0.0, self.arena_properties.height) as f32;

                                    player.path_target = Some((random_x, random_y, 0.0));

                                    player.path_cooldown = player.path_cooldown_reset;
                                }
                            }
                            else if player.bot_mode == BotMode::RunBlind {
                                player.path_target = None;
                            }
                            else {
                                player.path_target = None;
                            }


                            if let Some(dist_to_closest_vehicle) = vehicle.dist_to_closest_vehicle {
                                if (vehicle.health.value < vehicle.health.max ||
                                        (vehicle.shield.max > 0.0 && vehicle.shield.value == 0.0)) && 
                                        dist_to_closest_vehicle > BOT_DISENGAGE_DISTANCE &&
                                        player.last_hit_timer > 1.0 {
                                    player.bot_mode = BotMode::Repairing;
                                }
                            }

                            if let Some(path_plan) = player.path_plan.clone() {
                                if path_plan.len() >= 2 {
                                    let first_target = path_plan[1];
                                    let path_x = first_target.0;
                                    let path_y = first_target.1;

                                    let x_diff = vehicle_x - path_x;
                                    let y_diff = vehicle_y - path_y;

                                    let dist = (x_diff.powi(2) + y_diff.powi(2)).sqrt();

                                    let sq_vel = vehicle.dx.powi(2) + vehicle.dy.powi(2);
                                    let abs_vel = sq_vel.sqrt()*10.0; //Why does velocity not seem accurate without this weird multiplier???

                                    let approx_t_to_arrival = dist / abs_vel;
                                    let approx_t_to_rest = abs_vel / thrust_friction_decel_rate;

                                    let target_angle = y_diff.atan2(x_diff) + (PI / 2.0); //rotate by PI/2 to line up with 0deg is pointed towards top

                                    let mut angle_diff = vehicle_angle - target_angle;

                                    if angle_diff > PI {
                                        angle_diff = -(2.0 * PI - angle_diff);
                                    } else if angle_diff < -PI {
                                        angle_diff = -(-2.0 * PI - angle_diff);
                                    }

                                    let turn_value = 0.2;

                                    if angle_diff > 0.001 {
                                        vehicle_turn = Some(-turn_value);
                                    } else if angle_diff < -0.001 {
                                        vehicle_turn = Some(turn_value);
                                    } else {
                                        vehicle_turn = Some(0.0);
                                    }

                                    if angle_diff.abs() < 0.2 {
                                        if approx_t_to_arrival < approx_t_to_rest {
                                            vehicle_accel = Some(0.5);
                                        }
                                        else {
                                            vehicle_accel = Some(0.8);
                                        }
                                    }
                                }
                                else {
                                    vehicle_accel = Some(0.0);
                                    vehicle_turn = Some(0.0);
                                }                                
                            }
                            else { //random movement input when no path is specified
                                if player.bot_move_cooldown < 0.0 {
                                    //issue new move command
                                    vehicle_accel = Some(rng.gen_range(0.3, 0.5) as f32);
                                    vehicle_turn = Some(rng.gen_range(-0.3, 0.3) as f32);
                                }
                                else {
                                    //hold previous random move
                                    vehicle_accel = player.last_accel_input;
                                    vehicle_turn = player.last_turn_input;
                                }
                            }
                            player.last_accel_input = vehicle_accel;
                            player.last_turn_input = vehicle_turn;

                            if player.bot_move_cooldown < 0.0 {
                                player.bot_move_cooldown = player.bot_move_cooldown_reset;
                            }
                        }
                    } else if player.bot_mode == BotMode::StopAim
                        || player.bot_mode == BotMode::StrafeAim
                        || player.bot_mode == BotMode::Chasing
                        || player.bot_mode == BotMode::Swording
                    {
                        player.last_made_hit_timer += dt;

                        let continue_with_attacking_mode;

                        if let Some(dist_to_closest_vehicle) = vehicle.dist_to_closest_vehicle {

                            //if the closest vehicle is too far away to engage
                            if dist_to_closest_vehicle > BOT_DISENGAGE_DISTANCE
                                && player.bot_move_cooldown < 0.0
                            {
                                continue_with_attacking_mode = false;

                                player.bot_move_cooldown = player.bot_move_cooldown_reset;

                                let run_or_chase = rng.gen::<bool>();

                                if run_or_chase {
                                    player.bot_mode = BotMode::RunRandom;
                                    debug!("{} Run Random", player.id);
                                } else {
                                    player.bot_mode = BotMode::Chasing;
                                    debug!("{} Chasing", player.id);
                                    player.last_made_hit_timer = 0.0;
                                }
                            } else { //closest vehicle is close enough to engage
                                //vehicle is close, but the bot isn't hitting it within 3 seconds
                                if player.last_made_hit_timer > 3.0 && player.bot_mode != BotMode::Swording {
                                    continue_with_attacking_mode = false;

                                    player.bot_mode = BotMode::RunRandom;
                                    player.bot_move_cooldown = BOT_NO_HIT_MOVE_COOLDOWN;
                                }
                                else {
                                    //closest vehicle is out of current weapon range though
                                    if weapon_array.installed.len() > 0 { 
                                        let primary_weapon = &weapon_array.installed[PRIMARY_WEAPON_INDEX].weapon;
                                        if dist_to_closest_vehicle > primary_weapon.range_calc {
                                            player.bot_mode = BotMode::Chasing;
                                            debug!("{} Chasing", player.id);
                                            player.last_made_hit_timer = 0.0;
                                        }
                                    }
                                    continue_with_attacking_mode = true;
                                }
                            }
                        } else { //no closest vehicle exists, only vehicle alive right now?
                            continue_with_attacking_mode = false;

                            player.bot_move_cooldown = player.bot_move_cooldown_reset;

                            let run_or_chase = rng.gen::<bool>();

                            if run_or_chase {
                                player.bot_mode = BotMode::RunRandom;
                                debug!("{} Run Random", player.id);
                            } else {
                                player.bot_mode = BotMode::Chasing;
                                debug!("{} Chasing", player.id);
                                player.last_made_hit_timer = 0.0;
                            }
                        }
                        

                        if continue_with_attacking_mode {
                            //continue with Attacking mode
                            if player.bot_mode == BotMode::StopAim && player.last_hit_timer < 2.0 {
                                player.bot_mode = BotMode::StrafeAim;
                                debug!("{} StrafeAim {}", player.id, player.bot_move_cooldown);
                            }

                            if player.bot_move_cooldown < 0.0 {
                                if player.bot_mode == BotMode::StrafeAim {
                                    let left_or_right_strafe = rng.gen::<bool>();

                                    player.bot_move_cooldown = player.bot_move_cooldown_reset;
                                    if left_or_right_strafe {
                                        vehicle_strafe = Some(0.8);
                                    }
                                    else {
                                        vehicle_strafe = Some(-0.8);
                                    }
                                    
                                }
                                else {
                                    player.bot_move_cooldown = player.bot_move_cooldown_reset;
                                    vehicle_strafe = Some(0.0);
                                }
                            }

                            if let Some(attack_angle) = vehicle.angle_to_closest_vehicle {
                                let turn_value = 1.0;

                                if weapon_array.installed.len() > 0 {
                                    let primary_weapon = &weapon_array.installed[PRIMARY_WEAPON_INDEX].weapon;
                                    //Prepare magnitude of Turning and Acceleration input
                                    if player.bot_mode == BotMode::Swording {
                                        if primary_weapon.stats.fire_stats.mount_angle_special_offset > PI / 2.0
                                            || primary_weapon.stats.fire_stats.mount_angle_special_offset < -PI / 2.0
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
                                        vehicle_angle + primary_weapon.stats.fire_stats.mount_angle_special_offset - attack_angle;

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

                        if game_mode_setup.game_mode == GameModes::Race {
                            player.bot_mode = BotMode::Racing;
                            debug!("{} Racing", player.id);
                        }
                        else if player.bot_move_cooldown < 0.0 {
                            player.bot_mode = BotMode::RunRandom;
                            debug!("{} Run Random", player.id);
                        }
                    }
                }
            }

            let veh_x_comp = -vehicle_angle.sin(); //left is -, right is +
            let veh_y_comp = vehicle_angle.cos(); //up is +, down is -

            let veh_x_strafe_comp = -(vehicle_angle + PI/2.0).sin();
            let veh_y_strafe_comp = (vehicle_angle + PI/2.0).cos();


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
            if vehicle.stuck_accel_effect_timer > 0.0 {
                vehicle.stuck_accel_effect_timer -= dt;
            }

            if vehicle.state == VehicleState::Active {
                if let Some(move_amount) = vehicle_accel {
                    let scaled_amount: f32 = if vehicle.repair.activated {
                        0.0 as f32
                    } else if vehicle.malfunction > 0.0 {
                        thrust_accel_rate * move_amount * (100.0-vehicle.malfunction) as f32
                    } else if vehicle.stuck_accel_effect_timer > 0.0 {
                        thrust_accel_rate
                    } else if move_amount > 0.0 {
                        thrust_accel_rate * move_amount as f32
                    } else {
                        thrust_decel_rate * move_amount as f32
                    };

                    vehicle.dx += scaled_amount * veh_x_comp * dt;
                    vehicle.dy += scaled_amount * veh_y_comp * dt;

                    let position = Vector3::new(
                        vehicle_x - veh_x_comp*vehicle.height/2.0,
                        vehicle_y - veh_y_comp*vehicle.height/2.0, 
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

            //Update vehicle side strafing from strafing input
            if vehicle.movement_type == VehicleMovementType::Hover &&
                    vehicle.state == VehicleState::Active {
                if let Some(strafe_amount) = vehicle_strafe {
                    let scaled_amount: f32 = if vehicle.repair.activated {
                        0.0 as f32
                    } else if vehicle.malfunction > 0.0 {
                        thrust_strafe_accel_rate * strafe_amount * (100.0-vehicle.malfunction) as f32
                    } else {
                        thrust_strafe_accel_rate * strafe_amount as f32
                    };

                    vehicle.dx += scaled_amount * veh_x_strafe_comp * dt;
                    vehicle.dy += scaled_amount * veh_y_strafe_comp * dt;
                }
            }


            let sq_vel = vehicle.dx.powi(2) + vehicle.dy.powi(2);
            let abs_vel = sq_vel.sqrt();


            //Apply friction
            //this needs to be applied to vehicle momentum angle, not vehicle_angle angle
            let velocity_angle = vehicle.dy.atan2(vehicle.dx) - (PI / 2.0); //rotate by PI/2 to line up with vehicle_angle angle
            let velocity_x_comp = -velocity_angle.sin(); //left is -, right is +
            let velocity_y_comp = velocity_angle.cos(); //up is +, down is -

            let compare_velocity_angle;
            if abs_vel >= 0.001 {
                compare_velocity_angle = velocity_angle;
            }
            else {
                compare_velocity_angle = vehicle_angle; //no velocity = no slip
            }

            let mut slip_angle = vehicle_angle - compare_velocity_angle;

            if slip_angle > PI {
                slip_angle = -(2.0 * PI - slip_angle);
            } else if slip_angle < -PI {
                slip_angle = 2.0 * PI + slip_angle;
            }

            let slip_pct = 1.0 - ((slip_angle.abs() - PI/2.0).abs() / (PI/2.0));

            log::debug!("{} {} {} {}", velocity_angle, vehicle_angle, slip_angle, slip_pct);


            if vehicle.movement_type == VehicleMovementType::Hover {
                vehicle.dx -= thrust_friction_decel_rate * velocity_x_comp * dt;
                vehicle.dy -= thrust_friction_decel_rate * velocity_y_comp * dt;
            }
            else if vehicle.movement_type == VehicleMovementType::Car {
                // let veh_x_comp = -vehicle_angle.sin(); //left is -, right is +
                // let veh_y_comp = vehicle_angle.cos(); //up is +, down is -
    
                // let veh_x_strafe_comp = -(vehicle_angle + PI/2.0).sin();
                // let veh_y_strafe_comp = (vehicle_angle + PI/2.0).cos();

                // let tire_longitudinal_friction_decel_rate: f32 = 0.5;
                // let tire_lateral_friction_decel_rate: f32 = 2.0;

                vehicle.dx -= tire_longitudinal_friction_decel_rate * velocity_x_comp * (1.0 - slip_pct) * dt;
                vehicle.dy -= tire_longitudinal_friction_decel_rate * velocity_y_comp * (1.0 - slip_pct) * dt;

                vehicle.dx -= tire_lateral_friction_decel_rate * velocity_x_comp * slip_pct * dt;
                vehicle.dy -= tire_lateral_friction_decel_rate * velocity_y_comp * slip_pct * dt;
            }
            else if vehicle.movement_type == VehicleMovementType::Tank {

                vehicle.dx -= tank_track_longitudinal_friction_decel_rate * velocity_x_comp * (1.0 - slip_pct) * dt;
                vehicle.dy -= tank_track_longitudinal_friction_decel_rate * velocity_y_comp * (1.0 - slip_pct) * dt;

                vehicle.dx -= tank_track_lateral_friction_decel_rate * velocity_x_comp * (slip_pct) * dt;
                vehicle.dy -= tank_track_lateral_friction_decel_rate * velocity_y_comp * (slip_pct) * dt;
            }
            



            
            //Apply vehicle slow down effect
            if vehicle.restricted_velocity_timer <= 0.0 {
                //restore max velocity to unrestricted
                vehicle.restricted_max_velocity = vehicle.max_velocity;
            }
            else {
                vehicle.restricted_velocity_timer -= dt;
            }

            if abs_vel > vehicle.restricted_max_velocity {
                vehicle.dx *= vehicle.restricted_max_velocity / abs_vel;
                vehicle.dy *= vehicle.restricted_max_velocity / abs_vel;
            }


            //Transform on vehicle velocity
            if vehicle.dx.abs() > 0.1 {
                transform.prepend_translation_x(vehicle.dx * dt);
            }

            if vehicle.dy.abs() > 0.1 {
                transform.prepend_translation_y(vehicle.dy * dt);
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

                    let turnable;
                    if vehicle.movement_type == VehicleMovementType::Car {
                        if abs_vel >= 0.01 {
                            turnable = true;
                        }
                        else {
                            turnable = false;
                        }
                    }
                    else {
                        turnable = true;
                    }


                    if turnable {
                        if scaled_amount > 0.1 || scaled_amount < -0.1 {
                            if vehicle.dr > 1.0 {
                                vehicle.dr += (scaled_amount - rotate_friction_decel_rate) * dt;
                            } else if vehicle.dr < -1.0 {
                                vehicle.dr += (scaled_amount + rotate_friction_decel_rate) * dt;
                            } else {
                                vehicle.dr += (scaled_amount) * dt;
                            }
                        } else if vehicle.dr > 1.0 {
                            vehicle.dr += (-rotate_friction_decel_rate) * dt;
                        } else if vehicle.dr < -1.0 {
                            vehicle.dr += (rotate_friction_decel_rate) * dt;
                        } else {
                            vehicle.dr = 0.0;
                        }
                    }
                    else {
                        vehicle.dr = 0.0
                    }


                    vehicle.dr = vehicle.dr.min(2.5).max(-2.5);

                    transform.set_rotation_2d(vehicle_angle + vehicle.dr*dt);
                }
            }

            //Wall-collision logic
            let veh_rect_width = vehicle.height * 0.5 * veh_x_comp.abs()
                + vehicle.width * 0.5 * (1.0 - veh_x_comp.abs());
            let veh_rect_height = vehicle.height * 0.5 * veh_y_comp.abs()
                + vehicle.width * 0.5 * (1.0 - veh_y_comp.abs());

            let mut x_collision = false;
            let mut y_collision = false;

            if vehicle_x > (self.arena_properties.width - veh_rect_width) {
                //hit the right wall
                transform.set_translation_x(self.arena_properties.width - veh_rect_width);
                x_collision = true;
            } else if vehicle_x < (veh_rect_width) {
                //hit the left wall
                transform.set_translation_x(veh_rect_width);
                x_collision = true;
            }

            if vehicle_y > (self.arena_properties.height - veh_rect_height) {
                //hit the top wall
                transform.set_translation_y(self.arena_properties.height - veh_rect_height);
                y_collision = true;
            } else if vehicle_y < (veh_rect_height) {
                //hit the bottom wall
                transform.set_translation_y(veh_rect_height);
                y_collision = true;
            }

            if x_collision {
                vehicle.dx *= wall_hit_bounce_decel_pct * velocity_x_comp.abs();
                vehicle.dy *= wall_hit_non_bounce_decel_pct * velocity_y_comp.abs();

                if vehicle.state == VehicleState::Active {
                    if vehicle.collision_cooldown_timer <= 0.0 {
                        let damage: f32 = BASE_COLLISION_DAMAGE * abs_vel/100.0 * velocity_x_comp.abs();
                        debug!("Player {} has collided with {} damage", player.id, damage);

                        let vehicle_destroyed: bool = vehicle_damage_model(
                            vehicle,
                            None,
                            None,
                            damage,
                            COLLISION_PIERCING_DAMAGE_PCT,
                            COLLISION_SHIELD_DAMAGE_PCT,
                            COLLISION_ARMOR_DAMAGE_PCT,
                            COLLISION_HEALTH_DAMAGE_PCT,
                            DurationDamage::default(),
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

                        if abs_vel > 75.0 {
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
                        let damage: f32 = BASE_COLLISION_DAMAGE * abs_vel/100.0 * velocity_y_comp.abs();
                        debug!("Player {} has collided with {} damage", player.id, damage);

                        let vehicle_destroyed: bool = vehicle_damage_model(
                            vehicle,
                            None,
                            None,
                            damage,
                            COLLISION_PIERCING_DAMAGE_PCT,
                            COLLISION_SHIELD_DAMAGE_PCT,
                            COLLISION_ARMOR_DAMAGE_PCT,
                            COLLISION_HEALTH_DAMAGE_PCT,
                            DurationDamage::default(),
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

                        if abs_vel > 75.0 {
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

        for (player, vehicle, mut weapon_array, transform) in
            (&mut players, &mut vehicles, &mut weapon_arrays, &transforms).join()
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

            player.on_hill = false; //reset

            for (hitbox_entity, arena_element, hitbox_transform) in
                (&*entities, &arena_elements, &transforms).join()
            {
                let hitbox_x = hitbox_transform.translation().x;
                let hitbox_y = hitbox_transform.translation().y;

                let hit;
                let mut contact_data = None;

                if arena_element.hitbox.shape == HitboxShape::Circle {
                    let hitbox_collider_shape = Ball::new(arena_element.hitbox.width / 2.0);
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
                } else if arena_element.hitbox.shape == HitboxShape::Rectangle {
                    let hitbox_collider_shape = Cuboid::new(Vector2::new(
                        arena_element.hitbox.width/2.0, arena_element.hitbox.height/2.0)
                    );
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
                } else if arena_element.hitbox.shape == HitboxShape::InnerQuarterCircle {
                    let hitbox_collider_shape = Cuboid::new(Vector2::new(
                        arena_element.hitbox.width/2.0, arena_element.hitbox.height/2.0)
                    );
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
                } else if arena_element.hitbox.shape == HitboxShape::OuterQuarterCircle {
                    let hitbox_collider_shape = Cuboid::new(Vector2::new(
                        arena_element.hitbox.width/2.0, arena_element.hitbox.height/2.0)
                    );
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
                    if arena_element.obstacle_type == ObstacleType::Wall {
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
                            let damage: f32 = BASE_COLLISION_DAMAGE * abs_vel/100.0;
                            debug!("Player {} has collided with {} damage", player.id, damage);

                            let vehicle_destroyed: bool = vehicle_damage_model(
                                vehicle,
                                None,
                                None,
                                damage,
                                COLLISION_PIERCING_DAMAGE_PCT,
                                COLLISION_SHIELD_DAMAGE_PCT,
                                COLLISION_ARMOR_DAMAGE_PCT,
                                COLLISION_HEALTH_DAMAGE_PCT,
                                DurationDamage::default(),
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
                    }
                    else if arena_element.obstacle_type == ObstacleType::Zone {
                        if let Some(zone_effects) = arena_element.effects {
                            if zone_effects.damage_rate.abs() >= 0.001 {
                                let vehicle_destroyed: bool = vehicle_damage_model(
                                    vehicle,
                                    None,
                                    None,
                                    zone_effects.damage_rate * dt,
                                    COLLISION_PIERCING_DAMAGE_PCT,
                                    COLLISION_SHIELD_DAMAGE_PCT,
                                    COLLISION_ARMOR_DAMAGE_PCT,
                                    COLLISION_HEALTH_DAMAGE_PCT,
                                    DurationDamage::default(),
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
                            }

                            if zone_effects.accel_rate.abs() >= 0.001 {
                                let sq_vel = vehicle.dx.powi(2) + vehicle.dy.powi(2);
                                let abs_vel = sq_vel.sqrt();
                                
                                let delta_v = zone_effects.accel_rate * dt;
                                let new_abs_vel = abs_vel + delta_v;       

                                vehicle.dx *= new_abs_vel / abs_vel;
                                vehicle.dy *= new_abs_vel / abs_vel;
                            }
                        }
                    }
                    else if arena_element.obstacle_type == ObstacleType::Open {
                        if vehicle.state == VehicleState::Active {
                            //Non-collision related actions can only occur on Active vehicles
                            if arena_element.is_weapon_box {
                                let _ = entities.delete(hitbox_entity);

                                let new_weapon_name;
                                if arena_element.weapon_names.is_none() {
                                    //get random weapon from global list
                                    new_weapon_name = get_random_weapon_name(&game_weapon_setup.random_weapon_spawn_chances);
                                }
                                else {
                                    //get random weapon based on special chances list just for this weapon spawner
                                    new_weapon_name = get_random_weapon_name_build_chance(&arena_element.weapon_names);
                                }

                                if weapon_array.installed.len() >= 2 {
                                    let secondary_weapon = &weapon_array.installed[SECONDARY_WEAPON_INDEX].weapon;

                                    vehicle.weapon_weight -= secondary_weapon.stats.weight;

                                    weapon_icons_old_map.insert(player.id,
                                        (SECONDARY_WEAPON_INDEX, secondary_weapon.stats.weapon_fire_type.clone()));
                                }

                                update_weapon_properties(
                                    &mut weapon_array,
                                    SECONDARY_WEAPON_INDEX,
                                    1,
                                    None,
                                    new_weapon_name,
                                    &weapon_store_resource,
                                    &entities,
                                    &weapon_fire_resource,
                                    player.id,
                                    &lazy_update,
                                );

                                if weapon_array.installed.len() >= 2 {
                                    let new_secondary_weapon = &weapon_array.installed[SECONDARY_WEAPON_INDEX].weapon;
                                    vehicle.weapon_weight += new_secondary_weapon.stats.weight;
                                }
                            } else if arena_element.is_hill {
                                players_on_hill.push(player.id.clone());
                                player.on_hill = true;

                                let (r, g, b) = match player.id.clone() {
                                    0 => (1.0, 0.3, 0.3),
                                    1 => (0.3, 0.3, 1.0),
                                    2 => (0.3, 1.0, 0.3),
                                    3 => (1.0, 0.8, 0.3),
                                    _ => (1.0, 1.0, 1.0),
                                };

                                color_for_hill.push((r, g, b));
                            } else if (arena_element.checkpoint == RaceCheckpointType::Checkpoint)
                                    && (arena_element.checkpoint_id == player.checkpoint_completed + 1)
                            {
                                player.checkpoint_completed = arena_element.checkpoint_id;
                                debug!("{} checkpoints:{}", player.id, player.checkpoint_completed);
                            }
                            else if arena_element.checkpoint == RaceCheckpointType::Lap {
                                if player.checkpoint_completed == game_mode_setup.checkpoint_count-1 {
                                    player.laps_completed += 1;
                                }
                                player.checkpoint_completed = 0;
                                debug!("{} checkpoints:{}", player.id, player.checkpoint_completed);
                                debug!("{} laps:{}", player.id, player.laps_completed);
                            }
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

                transform.set_translation_x(vehicle_x - (contact_x - vehicle_x)/10. + vehicle.dx*dt);
                transform.set_translation_y(vehicle_y - (contact_y - vehicle_y)/10. + vehicle.dy*dt);
            }
        }

        //King of the Hill - Hill tint
        for (entity, _arena_element) in (&*entities, &arena_elements).join() {
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


        //Remove inactive Weapon UI Icons
        for (entity, player_icon) in (&*entities, &player_weapon_icons).join() {
            let weapon_icons_old = weapon_icons_old_map.get(&player_icon.player_id);

            if let Some(weapon_icons_old) = weapon_icons_old {
                let (weapon_id, weapon_fire_type) = weapon_icons_old;

                if *weapon_id == player_icon.weapon_id && *weapon_fire_type == player_icon.weapon_fire_type {
                    let _ = entities.delete(entity);
                }
            }
        }

        if self.rocket_spray_timer < 0.0 {
            self.rocket_spray_timer = ROCKET_SPRAY_COOLDOWN_RESET;
        }


        if DEBUG_LINES {
            for (arena_element, hitbox_transform) in
                (&arena_elements, &transforms).join()
            {
                let hitbox_x = hitbox_transform.translation().x;
                let hitbox_y = hitbox_transform.translation().y;

                debug_lines_resource.draw_line(
                    [hitbox_x - arena_element.hitbox.width/2.0, hitbox_y - arena_element.hitbox.height/2.0, 0.3].into(),
                    [hitbox_x - arena_element.hitbox.width/2.0, hitbox_y + arena_element.hitbox.height/2.0, 0.3].into(),
                    Srgba::new(0.7, 0.2, 0.2, 0.2),
                );

                debug_lines_resource.draw_line(
                    [hitbox_x - arena_element.hitbox.width/2.0, hitbox_y + arena_element.hitbox.height/2.0, 0.3].into(),
                    [hitbox_x + arena_element.hitbox.width/2.0, hitbox_y + arena_element.hitbox.height/2.0, 0.3].into(),
                    Srgba::new(0.7, 0.2, 0.2, 0.2),
                );

                debug_lines_resource.draw_line(
                    [hitbox_x + arena_element.hitbox.width/2.0, hitbox_y + arena_element.hitbox.height/2.0, 0.3].into(),
                    [hitbox_x + arena_element.hitbox.width/2.0, hitbox_y - arena_element.hitbox.height/2.0, 0.3].into(),
                    Srgba::new(0.7, 0.2, 0.2, 0.2),
                );

                debug_lines_resource.draw_line(
                    [hitbox_x + arena_element.hitbox.width/2.0, hitbox_y - arena_element.hitbox.height/2.0, 0.3].into(),
                    [hitbox_x - arena_element.hitbox.width/2.0, hitbox_y - arena_element.hitbox.height/2.0, 0.3].into(),
                    Srgba::new(0.7, 0.2, 0.2, 0.2),
                );
            }
        }
    }
}