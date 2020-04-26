use crate::entities::ui::PlayerStatusText;
use amethyst::core::Transform;
use amethyst::ecs::prelude::{Component, DenseVecStorage};

use rand::Rng;
use std::f32::consts::PI;

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};

pub const VEHICLE_HEIGHT: f32 = 12.0;
pub const VEHICLE_WIDTH: f32 = 6.0;

pub struct Vehicle {
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
    pub dr: f32,
    pub collision_cooldown_timer: f32,
    pub health: f32,
    pub health_max: f32,
    pub shield: f32,
    pub shield_max: f32,
    pub shield_recharge_rate: f32,
    pub shield_cooldown_timer: f32,
    pub shield_cooldown_reset: f32,
    pub armor: f32,
    pub armor_max: f32,
    pub weight: f32,
    pub engine_power: f32,
    pub respawn_timer: f32,
    pub in_respawn: bool,
    pub player_status_text: PlayerStatusText,
}

impl Component for Vehicle {
    type Storage = DenseVecStorage<Self>;
}

impl Vehicle {
    pub fn new(player_status_text: PlayerStatusText) -> Vehicle {
        Vehicle {
            width: VEHICLE_WIDTH,
            height: VEHICLE_HEIGHT,
            dx: 0.0,
            dy: 0.0,
            dr: 0.0,
            collision_cooldown_timer: -1.0,
            health: 100.0,
            health_max: 100.0,
            shield: 100.0,
            shield_max: 100.0,
            shield_recharge_rate: 5.0,
            shield_cooldown_timer: -1.0,
            shield_cooldown_reset: 10.0,
            armor: 100.0,
            armor_max: 100.0,
            weight: 100.0,
            engine_power: 100.0,
            respawn_timer: 5.0,
            in_respawn: false,
            player_status_text,
        }
    }
}

pub fn kill_restart_vehicle(vehicle: &mut Vehicle, transform: &mut Transform) {
    transform.set_translation_x(10.0 * ARENA_WIDTH);
    transform.set_translation_y(10.0 * ARENA_HEIGHT);

    vehicle.in_respawn = true;
}

pub fn check_respawn_vehicle(vehicle: &mut Vehicle, transform: &mut Transform, dt: f32) {
    let mut rng = rand::thread_rng();

    vehicle.respawn_timer -= dt;

    if vehicle.respawn_timer < 0.0 {
        vehicle.in_respawn = false;
        vehicle.respawn_timer = 5.0;

        vehicle.dx = 0.0;
        vehicle.dy = 0.0;
        vehicle.dr = 0.0;

        vehicle.shield = vehicle.shield_max;
        vehicle.shield_cooldown_timer = -1.;

        vehicle.armor = vehicle.armor_max;
        vehicle.health = vehicle.health_max;

        let spawn_index = rng.gen_range(0, 4);

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
