use amethyst::{
    core::transform::Transform,
    core::math::Vector3,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect, Component, DenseVecStorage},
    renderer::{
        Transparent,
        SpriteRender,
        palette::Srgba,
        resources::Tint,
    },
};

use ron::de::from_reader;
use serde::Deserialize;
use std::{collections::HashMap, fs::File};
use std::f32::consts::PI;

use crate::components::{PlayerWeaponIcon};
use crate::rally::UI_HEIGHT;
use crate::resources::WeaponFireResource;


//pub const WEAPON_CONFIGS: HashMap<WeaponNames, WeaponStats> = build_weapon_store();



#[derive(Clone, Debug, PartialEq, Deserialize, Hash, Eq)]
pub enum WeaponNames {
    LaserBeam,
    LaserPulse,
    LaserDouble,
    ProjectileRapidFire,
    ProjectileBurstFire,
    ProjectileCannonFire,
    Mine,
    Missile,
    Rockets,
    LaserSword,
    LaserDoubleBurst,
    RocketGrenade,
    ProjectileSteadyFire,
}


//For Gun-Game mode rules
pub fn get_next_weapon_name(weapon_name: WeaponNames) -> Option<WeaponNames> {
    match weapon_name {
        WeaponNames::LaserDouble => Some(WeaponNames::ProjectileRapidFire),
        WeaponNames::ProjectileRapidFire => Some(WeaponNames::Missile),
        WeaponNames::Missile => Some(WeaponNames::LaserBeam),
        WeaponNames::LaserBeam => Some(WeaponNames::ProjectileCannonFire),
        WeaponNames::ProjectileCannonFire => Some(WeaponNames::LaserPulse),
        WeaponNames::LaserPulse => Some(WeaponNames::Rockets),
        WeaponNames::Rockets => Some(WeaponNames::ProjectileBurstFire),
        WeaponNames::ProjectileBurstFire => Some(WeaponNames::Mine),
        WeaponNames::Mine => Some(WeaponNames::LaserDoubleBurst),
        WeaponNames::LaserDoubleBurst => Some(WeaponNames::RocketGrenade),
        WeaponNames::RocketGrenade => Some(WeaponNames::ProjectileSteadyFire),
        WeaponNames::ProjectileSteadyFire => Some(WeaponNames::LaserSword),
        WeaponNames::LaserSword => None,
    }
}



#[derive(Copy, Clone, Debug, Deserialize)]
pub struct WeaponStats {
    pub weapon_type: WeaponTypes,
    pub heat_seeking: bool,
    pub heat_seeking_agility: f32,
    pub attached: bool,
    pub deployed: bool,
    pub tracking_angle: f32,
    pub cooldown_reset: f32,
    pub burst_shot_limit: u32,
    pub burst_cooldown_reset: f32,
    pub damage: f32,
    pub shot_speed: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
}

pub fn build_weapon_store() -> HashMap<WeaponNames, WeaponStats> {
    let input_path = format!("{}/config/weapons.ron", env!("CARGO_MANIFEST_DIR"));
    let f = File::open(&input_path).expect("Failed opening file");
    let weapon_configs: HashMap<WeaponNames, WeaponStats> = match from_reader(f) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to load config: {}", e);

            std::process::exit(1);
        }
    };

    //println!("Config: {:?}", &weapon_configs);

    weapon_configs
}



#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
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
    LaserSword,
}


#[derive(Clone)]
pub struct Weapon {
    pub name: WeaponNames,
    pub icon_entity: Entity,
    pub x: f32,
    pub y: f32,
    pub aim_angle: f32,
    pub stats: WeaponStats,
    pub cooldown_timer: f32,
    pub burst_shots: u32,
}

impl Component for Weapon {
    type Storage = DenseVecStorage<Self>;
}

impl Weapon {
    pub fn new(
        name: WeaponNames,
        icon_entity: Entity,
        stats: WeaponStats,
    ) -> Weapon {
        Weapon {
            name,
            icon_entity,
            x: 0.0,
            y: 0.0,
            aim_angle: 0.0,
            stats,
            cooldown_timer: 0.0,
            burst_shots: 0,
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
    pub owner_player_id: usize,
    pub shot_speed: f32,
    pub damage: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
    pub heat_seeking: bool,
    pub heat_seeking_agility: f32,
    pub attached: bool,
    pub deployed: bool,
    pub weapon_type: WeaponTypes,
    pub weapon_name: WeaponNames,
}

impl Component for WeaponFire {
    type Storage = DenseVecStorage<Self>;
}

impl WeaponFire {
    pub fn new(
        weapon_name: WeaponNames,
        weapon_type: WeaponTypes,
        owner_player_id: usize,
        heat_seeking: bool,
        heat_seeking_agility: f32,
        attached: bool,
        deployed: bool,
        shot_speed: f32,
        damage: f32,
        shield_damage_pct: f32,
        armor_damage_pct: f32,
        piercing_damage_pct: f32,
        health_damage_pct: f32,
    ) -> WeaponFire {
        let (width, height) = match weapon_type.clone() {
            WeaponTypes::LaserDouble => (3.0, 6.0),
            WeaponTypes::LaserBeam => (1.0, 12.0),
            WeaponTypes::LaserPulse => (1.0, 3.0),
            WeaponTypes::ProjectileBurstFire => (1.0, 4.0),
            WeaponTypes::ProjectileRapidFire => (1.0, 2.0),
            WeaponTypes::ProjectileCannonFire => (2.0, 3.0),
            WeaponTypes::Missile => (3.0, 5.0),
            WeaponTypes::Rockets => (5.0, 3.0),
            WeaponTypes::Mine => (3.0, 3.0),
            WeaponTypes::LaserSword => (3.0, 4.0),
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
            shot_speed: shot_speed,
            shield_damage_pct: shield_damage_pct,
            armor_damage_pct: armor_damage_pct,
            piercing_damage_pct: piercing_damage_pct,
            health_damage_pct: health_damage_pct,
            heat_seeking,
            heat_seeking_agility,
            attached,
            deployed,
            weapon_name,
            weapon_type,
        }
    }
}




pub fn update_weapon_properties(weapon: &mut Weapon, weapon_name: WeaponNames) {
    weapon.name = weapon_name.clone();
    weapon.stats = build_named_weapon(weapon_name);
}


pub fn build_named_weapon(
    weapon_name: WeaponNames,
) -> WeaponStats {

    let WEAPON_CONFIGS: HashMap<WeaponNames, WeaponStats> = build_weapon_store();

    if let weapon_config = Some(WEAPON_CONFIGS.get(&weapon_name)) {
        *weapon_config.unwrap().unwrap()
    }
    else {
        WeaponStats {
            weapon_type: WeaponTypes::LaserDouble,
            heat_seeking: false,
            heat_seeking_agility: 0.0,
            attached: false,
            deployed: false,
            tracking_angle: 0.0,
            cooldown_reset: 100.0,
            burst_shot_limit: 0,
            burst_cooldown_reset: 100.0,
            damage: 0.0,
            shot_speed: 0.0,
            shield_damage_pct: 0.0,
            armor_damage_pct: 0.0,
            piercing_damage_pct: 0.0,
            health_damage_pct: 0.0,
        }
    }
}


pub fn update_weapon_icon(
    entities: &Entities,
    weapon: &mut Weapon,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    player_id: usize,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let weapon_type = weapon.stats.weapon_type.clone();

    //UI icon
    let weapon_entity: Entity = entities.create();

    weapon.icon_entity = weapon_entity;

    let x = 15.;
    let y = UI_HEIGHT - 10.;
    let dx = 32.;
    let dx2 = 4.;

    let weapon_icon_dx = 70.0;

    let (icon_scale, mut weapon_sprite) = match weapon_type.clone() {
        WeaponTypes::LaserDouble => (3.0, weapon_fire_resource.laser_double_sprite_render.clone()),
        WeaponTypes::LaserBeam => (1.0, weapon_fire_resource.laser_beam_sprite_render.clone()),
        WeaponTypes::LaserPulse => (3.0, weapon_fire_resource.laser_burst_sprite_render.clone()),
        WeaponTypes::ProjectileBurstFire => {
            (3.0, weapon_fire_resource.projectile_burst_render.clone())
        }
        WeaponTypes::ProjectileRapidFire => {
            (3.0, weapon_fire_resource.projectile_rapid_render.clone())
        }
        WeaponTypes::ProjectileCannonFire => (
            3.0,
            weapon_fire_resource.projectile_cannon_sprite_render.clone(),
        ),
        WeaponTypes::Missile => (2.0, weapon_fire_resource.missile_sprite_render.clone()),
        WeaponTypes::Rockets => (2.0, weapon_fire_resource.rockets_sprite_render.clone()),
        WeaponTypes::Mine => (2.0, weapon_fire_resource.mine_p1_sprite_render.clone()),
        WeaponTypes::LaserSword => (1.0, weapon_fire_resource.laser_sword_sprite_render.clone()),
    };

    if weapon_type.clone() == WeaponTypes::Mine {
        weapon_sprite = get_mine_sprite(player_id, weapon.stats.shot_speed.clone(), weapon_fire_resource);
    }

    let mut icon_weapon_transform = Transform::default();

    let starting_x = match player_id {
        0 => (x),
        1 => (x + 3.0 * dx + dx2),
        2 => (x + 6.0 * dx + 2.0 * dx2),
        3 => (x + 9.0 * dx + 3.0 * dx2),
        _ => (0.0),
    };

    icon_weapon_transform.set_translation_xyz((starting_x + weapon_icon_dx) as f32, y, 0.0);
    icon_weapon_transform.set_rotation_2d(-PI / 2.0);
    icon_weapon_transform.set_scale(Vector3::new(icon_scale, icon_scale, 0.0));

    let icon_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

    lazy_update.insert(weapon_entity, PlayerWeaponIcon::new(player_id, weapon_type));
    lazy_update.insert(weapon_entity, weapon_sprite);
    lazy_update.insert(weapon_entity, icon_weapon_transform);
    lazy_update.insert(weapon_entity, icon_tint);
    lazy_update.insert(weapon_entity, Transparent);
}


pub fn get_mine_sprite(player_id: usize,
        shot_speed: f32,
        weapon_fire_resource: &WeaponFireResource,
    ) -> SpriteRender
{
    if shot_speed > 0.0 {
        weapon_fire_resource.mine_neutral_sprite_render.clone()
    }
    else {
        match player_id {
            0 => weapon_fire_resource.mine_p1_sprite_render.clone(),
            1 => weapon_fire_resource.mine_p2_sprite_render.clone(),
            2 => weapon_fire_resource.mine_p3_sprite_render.clone(),
            3 => weapon_fire_resource.mine_p4_sprite_render.clone(),
            _ => weapon_fire_resource.mine_neutral_sprite_render.clone(),
        }
    }
}
