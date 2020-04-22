use amethyst::ecs::prelude::{Component, DenseVecStorage};

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
    pub shield: f32,
    pub shield_max: f32,
    pub shield_recharge_rate: f32,
    pub shield_cooldown_timer: f32,
    pub shield_cooldown_reset: f32,
    pub armor: f32,
    pub weight: f32,
    pub engine_power: f32
}

impl Component for Vehicle {
    type Storage = DenseVecStorage<Self>;
}

impl Vehicle {
    pub fn new() -> Vehicle {
        Vehicle {
            width: VEHICLE_WIDTH,
            height: VEHICLE_HEIGHT,
            dx: 0.0,
            dy: 0.0,
            dr: 0.0,
            collision_cooldown_timer: -1.0,
            health: 100.0,
            shield: 100.0,
            shield_max: 100.0,
            shield_recharge_rate: 5.0,
            shield_cooldown_timer: -1.0,
            shield_cooldown_reset: 10.0,
            armor: 200.0,
            weight: 100.0,
            engine_power: 100.0,
        }
    }
}