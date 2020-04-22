use amethyst::ecs::prelude::{Component, DenseVecStorage};

pub fn weapon_type_from_u8(n: u8) -> WeaponTypes {
    match n {
        0 => WeaponTypes::LaserBeam,
        1 => WeaponTypes::LaserPulse,
        2 => WeaponTypes::LaserDouble,
        3 => WeaponTypes::ProjectileRapidFire,
        4 => WeaponTypes::ProjectileBurstFire,
        5 => WeaponTypes::ProjectileCannonFire,
        6 => WeaponTypes::Mine,
        7 => WeaponTypes::Missile,
        8 => WeaponTypes::Rockets,
        _ => WeaponTypes::LaserBeam,
    }
}


#[derive(PartialEq)]
#[derive(Clone)]
pub enum WeaponTypes {
    LaserBeam,
    LaserPulse,
    LaserDouble,
    ProjectileRapidFire,
    ProjectileBurstFire,
    ProjectileCannonFire,
    Mine,
    Missile,
    Rockets,
}


#[derive(Clone)]
pub struct Weapon {
    pub x: f32,
    pub y: f32,
    pub aim_angle: f32,
    pub weapon_type: WeaponTypes,
    pub weapon_cooldown_timer: f32,
    pub weapon_cooldown_reset: f32,
    pub burst_shots: u32,
    pub burst_shot_limit: u32,
    pub burst_cooldown_reset: f32,
    pub damage: f32,
    pub weapon_shot_speed: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
}

impl Component for Weapon {
    type Storage = DenseVecStorage<Self>;
}

impl Weapon {
    pub fn new(weapon_type: WeaponTypes, 
        weapon_cooldown: f32, 
        burst_shot_limit: u32, 
        burst_cooldown: f32,
        weapon_shot_speed: f32,
        damage: f32,
        shield_damage_pct: f32,
        armor_damage_pct: f32,
        piercing_damage_pct: f32,
        health_damage_pct: f32,
    ) -> Weapon {

        Weapon {
            x: 0.0,
            y: 0.0,
            aim_angle: 0.0,
            weapon_type,
            weapon_cooldown_timer: -1.0,
            weapon_cooldown_reset: weapon_cooldown,
            burst_shots: 0,
            burst_shot_limit: burst_shot_limit,
            burst_cooldown_reset: burst_cooldown,
            damage: damage,
            weapon_shot_speed: weapon_shot_speed,
            shield_damage_pct: shield_damage_pct,
            armor_damage_pct: armor_damage_pct,
            piercing_damage_pct: piercing_damage_pct,
            health_damage_pct: health_damage_pct,
        }
    }
}


#[derive(Clone)]
pub struct WeaponFire {
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub spawn_angle: f32,
    pub weapon_shot_speed: f32,
    pub owner_player_id: usize,
    pub damage: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
    pub weapon_type: WeaponTypes,
}

impl Component for WeaponFire {
    type Storage = DenseVecStorage<Self>;
}


impl WeaponFire {
    pub fn new(weapon_type: WeaponTypes, 
        owner_player_id: usize,
        weapon_shot_speed: f32,
        damage: f32,
        shield_damage_pct: f32,
        armor_damage_pct: f32,
        piercing_damage_pct: f32,
        health_damage_pct: f32,
    ) -> WeaponFire {

        WeaponFire {
            width: 1.0,
            height: 1.0,
            dx: 0.0,
            dy: 0.0,
            spawn_x: 0.0,
            spawn_y: 0.0,
            spawn_angle: 0.0,
            owner_player_id,
            damage: damage,
            weapon_shot_speed: weapon_shot_speed,
            shield_damage_pct: shield_damage_pct,
            armor_damage_pct: armor_damage_pct,
            piercing_damage_pct: piercing_damage_pct,
            health_damage_pct: health_damage_pct,
            weapon_type,
        }
    }
}
