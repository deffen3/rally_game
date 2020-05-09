use crate::entities::ui::PlayerStatusText;
use amethyst::core::Transform;
use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};

use rand::Rng;
use std::f32::consts::PI;

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};
use crate::components::{Shield, Armor, Health, Repair, Player};
use crate::resources::{GameModes};

pub const VEHICLE_HEIGHT: f32 = 12.0;
pub const VEHICLE_WIDTH: f32 = 7.0;


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VehicleState {
    Active,
    InActive,
    InRespawn,
}


pub struct Vehicle {
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
    pub dr: f32,
    pub angle_to_closest_vehicle: Option<f32>,
    pub dist_to_closest_vehicle: Option<f32>,
    pub angle_to_closest_targetable_vehicle: Option<f32>,
    pub dist_to_closest_targetable_vehicle: Option<f32>,
    pub collision_cooldown_timer: f32,
    pub health: Health,
    pub armor: Armor,
    pub shield: Shield,
    pub repair: Repair,
    pub engine_weight: f32,
    pub weapon_weight: f32,
    pub engine_force: f32,
    pub max_velocity: f32,
    pub respawn_timer: f32,
    pub death_x: f32,
    pub death_y: f32,
    pub death_angle: f32,
    pub state: VehicleState,
    pub player_status_text: PlayerStatusText,
    
}

impl Component for Vehicle {
    type Storage = DenseVecStorage<Self>;
}

impl Vehicle {
    pub fn new(player_status_text: PlayerStatusText,
            health_entity: Entity,
            armor_entity: Entity,
            shield_entity: Entity,
            repair_entity: Entity,
            max_shield: f32,
            max_armor: f32,
            max_health: f32,
            engine_force: f32,
            engine_weight: f32,
            max_velocity: f32,
            weapon_weight: f32,
    ) -> Vehicle {
        Vehicle {
            width: VEHICLE_WIDTH,
            height: VEHICLE_HEIGHT,
            dx: 0.0,
            dy: 0.0,
            dr: 0.0,
            angle_to_closest_vehicle: None,
            dist_to_closest_vehicle: None,
            angle_to_closest_targetable_vehicle: None,
            dist_to_closest_targetable_vehicle: None,
            collision_cooldown_timer: -1.0,
            health: Health {
                value: max_health,
                max: max_health,
                repair_rate: 5.0,
                entity: health_entity,
            },
            armor: Armor {
                value: max_armor,
                max: max_armor,
                entity: armor_entity,
            },
            shield: Shield {
                value: max_shield,
                max: max_shield,
                recharge_rate: 5.0,
                cooldown_timer: -1.0,
                cooldown_reset: 3.5,
                repair_timer: 0.0,
                repair_threshold: 5.0,
                radius: 15.0,
                entity: shield_entity,
            },
            repair: Repair {
                activated: false,
                init_timer: 0.0,
                init_threshold: 2.5,
                entity: repair_entity,
            },
            engine_force,
            engine_weight,
            weapon_weight,
            max_velocity,
            respawn_timer: 5.0,
            death_x: 0.0,
            death_y: 0.0,
            death_angle: 0.0,
            state: VehicleState::Active,
            player_status_text,
        }
    }
}

pub fn kill_restart_vehicle(
        player: &Player, 
        vehicle: &mut Vehicle, 
        transform: &mut Transform,
        stock_lives: i32,
    ) {

    vehicle.death_x = transform.translation().x;
    vehicle.death_y = transform.translation().y;

    let (_, _, vehicle_angle) = transform.rotation().euler_angles();
    vehicle.death_angle = vehicle_angle;

    //transform.set_translation_x(10.0 * ARENA_WIDTH);
    //transform.set_translation_y(10.0 * ARENA_HEIGHT);

    if stock_lives > 0 && player.deaths >= stock_lives {
        vehicle.state = VehicleState::InActive;
    }
    else {
        vehicle.state = VehicleState::InRespawn;
    }    
}


pub fn check_respawn_vehicle(
        vehicle: &mut Vehicle,
        transform: &mut Transform,
        dt: f32,
        game_mode: GameModes,
        last_spawn_index: u32,
    ) -> u32 {
    let mut rng = rand::thread_rng();

    let mut spawn_index = last_spawn_index;

    if vehicle.state == VehicleState::InRespawn {
        vehicle.respawn_timer -= dt;

        if vehicle.respawn_timer < 0.0 {
            vehicle.state = VehicleState::Active;

            vehicle.respawn_timer = 5.0;

            vehicle.dx = 0.0;
            vehicle.dy = 0.0;
            vehicle.dr = 0.0;

            vehicle.shield.value = vehicle.shield.max;
            vehicle.shield.cooldown_timer = -1.;

            vehicle.armor.value = vehicle.armor.max;
            vehicle.health.value = vehicle.health.max;

            if game_mode == GameModes::Race {
                transform.set_rotation_2d(vehicle.death_angle);
                transform.set_translation_xyz(vehicle.death_x, vehicle.death_y, 0.0);
            }
            else {
                //Ensure that the spawn_index != last_spawn_index
                spawn_index = rng.gen_range(0, 3) as u32;

                if spawn_index >= last_spawn_index {
                    spawn_index += 1;
                }

                let spacing_factor = 5.0;
                let height = ARENA_HEIGHT + UI_HEIGHT;

                let (starting_rotation, starting_x, starting_y) = match spawn_index {
                    0 => (
                        -PI / 4.0,
                        ARENA_WIDTH / spacing_factor,
                        height / spacing_factor,
                    ),
                    1 => (
                        PI + PI / 4.0,
                        ARENA_WIDTH / spacing_factor,
                        height - (height / spacing_factor),
                    ),
                    2 => (
                        PI / 2.0 - PI / 4.0,
                        ARENA_WIDTH - (ARENA_WIDTH / spacing_factor),
                        height / spacing_factor,
                    ),
                    3 => (
                        PI / 2.0 + PI / 4.0,
                        ARENA_WIDTH - (ARENA_WIDTH / spacing_factor),
                        height - (height / spacing_factor),
                    ),
                    _ => (
                        -PI / 4.0,
                        ARENA_WIDTH / spacing_factor,
                        height / spacing_factor,
                    ),
                };

                transform.set_rotation_2d(starting_rotation as f32);
                transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.0);
            }
        }
    }

    spawn_index
}
