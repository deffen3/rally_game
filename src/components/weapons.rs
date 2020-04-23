use amethyst::ecs::prelude::{Component, DenseVecStorage};

pub fn weapon_type_from_u8(n: u8) -> WeaponTypes {
    match n {
        0 => WeaponTypes::LaserDouble,
        1 => WeaponTypes::ProjectileBurstFire,
        2 => WeaponTypes::Mine,
        3 => WeaponTypes::LaserPulse,
        4 => WeaponTypes::ProjectileRapidFire,
        5 => WeaponTypes::Rockets,
        6 => WeaponTypes::LaserBeam,
        7 => WeaponTypes::ProjectileCannonFire,
        8 => WeaponTypes::Missile,
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
    pub heat_seeking: bool,
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
        heat_seeking: bool,
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
            heat_seeking,
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
    pub heat_seeking: bool,
    pub weapon_type: WeaponTypes,
}

impl Component for WeaponFire {
    type Storage = DenseVecStorage<Self>;
}


impl WeaponFire {
    pub fn new(weapon_type: WeaponTypes, 
        owner_player_id: usize,
        heat_seeking: bool,
        weapon_shot_speed: f32,
        damage: f32,
        shield_damage_pct: f32,
        armor_damage_pct: f32,
        piercing_damage_pct: f32,
        health_damage_pct: f32,
    ) -> WeaponFire {

        let (width, height) = match weapon_type.clone()
        {                                      
            WeaponTypes::LaserDouble =>         (3.0, 6.0),
            WeaponTypes::LaserBeam =>           (1.0, 20.0),
            WeaponTypes::LaserPulse =>          (1.0, 3.0),
            WeaponTypes::ProjectileBurstFire => (1.0, 4.0),
            WeaponTypes::ProjectileRapidFire => (1.0, 2.0),
            WeaponTypes::ProjectileCannonFire =>(2.0, 3.0),
            WeaponTypes::Missile =>             (3.0, 5.0),
            WeaponTypes::Rockets =>             (5.0, 3.0),
            WeaponTypes::Mine =>                (3.0, 3.0),
        };

        WeaponFire {
            width,
            height,
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
            heat_seeking,
            weapon_type,
        }
    }
}
