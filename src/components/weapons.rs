use amethyst::{
    core::math::Vector3,
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entities, Entity, LazyUpdate, ReadExpect},
    renderer::{palette::Srgba, resources::Tint, SpriteRender, Transparent},
    utils::removal::Removal,
};

use rand::Rng;
use ron::de::from_reader;
use serde::Deserialize;
use std::f32::consts::PI;
use std::{collections::HashMap, fs::File};

use crate::components::PlayerWeaponIcon;
use crate::rally::UI_HEIGHT;
use crate::resources::{GameModeSetup, WeaponFireResource};

//pub const WEAPON_CONFIGS: HashMap<WeaponNames, WeaponStats> = build_weapon_store();

#[derive(Clone, Debug, PartialEq, Deserialize, Hash, Eq)]
pub enum WeaponNames {
    LaserBeam,
    LaserPulse,
    LaserPulseGimballed,
    LaserDouble,
    LaserDoubleGimballed,
    LaserDoubleBurstSide,
    ProjectileRapidFire,
    ProjectileRapidFireTurret,
    ProjectileSteadyFire,
    ProjectileBurstFire,
    ProjectileCannonFire,
    Shotgun,
    Mine,
    Trap,
    Missile,
    Rockets,
    LaserSword,
    BackwardsLaserSword,
    SuperRocketGrenades,
    Flamethrower,
}

pub fn get_random_weapon_name(game_mode_setup: &ReadExpect<GameModeSetup>) -> WeaponNames {
    let mut rng = rand::thread_rng();
    let chance_selector = rng.gen_range(0.0, 1.0);

    let mut weapon_selector = game_mode_setup.starter_weapon.clone();

    for (weapon_name, chance) in game_mode_setup.weapon_spawn_chances.iter() {
        if *chance >= chance_selector {
            weapon_selector = weapon_name.clone();
            break;
        }
    }

    // match index {
    //     0 => WeaponNames::LaserDoubleGimballed,
    //     1 => WeaponNames::LaserDoubleGimballed,
    //     2 => WeaponNames::ProjectileRapidFireTurret,
    //     3 => WeaponNames::Flamethrower,
    //     4 => WeaponNames::Missile,
    //     5 => WeaponNames::LaserBeam,
    //     6 => WeaponNames::Shotgun,
    //     7 => WeaponNames::ProjectileCannonFire,
    //     8 => WeaponNames::LaserPulseGimballed,
    //     9 => WeaponNames::Rockets,
    //     10 => WeaponNames::ProjectileBurstFire,
    //     11 => WeaponNames::LaserDoubleBurstSide,
    //     12 => WeaponNames::SuperRocketGrenades,
    //     13 => WeaponNames::Mine,
    //     14 => WeaponNames::LaserSword,
    //     15 => WeaponNames::Trap,
    //     16 => WeaponNames::BackwardsLaserSword,
    //     17 => WeaponNames::ProjectileSteadyFire,
    //     18 => WeaponNames::LaserDouble,
    //     19 => WeaponNames::ProjectileRapidFire,
    //     20 => WeaponNames::LaserPulse,
    //     _ => WeaponNames::LaserDoubleGimballed,
    // }

    weapon_selector
}

//For Gun-Game mode rules
pub fn get_next_weapon_name(weapon_name: WeaponNames) -> Option<WeaponNames> {
    match weapon_name {
        WeaponNames::LaserDoubleGimballed => Some(WeaponNames::ProjectileRapidFireTurret),
        WeaponNames::ProjectileRapidFireTurret => Some(WeaponNames::Flamethrower),
        WeaponNames::Flamethrower => Some(WeaponNames::Missile),
        WeaponNames::Missile => Some(WeaponNames::LaserBeam),
        WeaponNames::LaserBeam => Some(WeaponNames::Shotgun),
        WeaponNames::Shotgun => Some(WeaponNames::ProjectileCannonFire),
        WeaponNames::ProjectileCannonFire => Some(WeaponNames::LaserPulseGimballed),
        WeaponNames::LaserPulseGimballed => Some(WeaponNames::Rockets),
        WeaponNames::Rockets => Some(WeaponNames::ProjectileBurstFire),
        WeaponNames::ProjectileBurstFire => Some(WeaponNames::LaserDoubleBurstSide),
        WeaponNames::LaserDoubleBurstSide => Some(WeaponNames::SuperRocketGrenades),
        WeaponNames::SuperRocketGrenades => Some(WeaponNames::Mine),
        WeaponNames::Mine => Some(WeaponNames::LaserSword),
        WeaponNames::LaserSword => Some(WeaponNames::Trap),
        WeaponNames::Trap => Some(WeaponNames::BackwardsLaserSword),
        WeaponNames::BackwardsLaserSword => None,
        WeaponNames::ProjectileSteadyFire => None,
        WeaponNames::LaserDouble => None,
        WeaponNames::ProjectileRapidFire => None,
        WeaponNames::LaserPulse => None,
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
    pub spread_angle: f32,
    pub mounted_angle: f32,
    pub cooldown_reset: f32,
    pub burst_shot_limit: u32,
    pub burst_cooldown_reset: f32,
    pub shot_life_limit: f32,
    pub damage: f32,
    pub shot_speed: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
    pub weight: f32,
}

pub fn build_weapon_store() -> HashMap<WeaponNames, WeaponStats> {
    let input_path = format!("{}/config/weapons.ron", env!("CARGO_MANIFEST_DIR"));
    let f = File::open(&input_path).expect("Failed opening file");

    from_reader(f).expect("Failed to load config")
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
    Trap,
    Missile,
    Rockets,
    LaserSword,
    Flame,
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
    pub fn new(name: WeaponNames, icon_entity: Entity, stats: WeaponStats) -> Weapon {
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
    pub shot_life_timer: f32,
    pub shot_life_limit: f32,
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
        spawn_angle: f32,
        shot_speed: f32,
        shot_life_limit: f32,
        damage: f32,
        shield_damage_pct: f32,
        armor_damage_pct: f32,
        piercing_damage_pct: f32,
        health_damage_pct: f32,
    ) -> WeaponFire {
        let (width, height) = match weapon_type {
            WeaponTypes::LaserDouble => (3.0, 6.0),
            WeaponTypes::LaserBeam => (1.0, 12.0),
            WeaponTypes::LaserPulse => (2.0, 5.0),
            WeaponTypes::ProjectileBurstFire => (1.0, 4.0),
            WeaponTypes::ProjectileRapidFire => (1.0, 2.0),
            WeaponTypes::ProjectileCannonFire => (3.0, 3.0),
            WeaponTypes::Missile => (5.0, 6.0),
            WeaponTypes::Rockets => (5.0, 4.0),
            WeaponTypes::Mine => (4.0, 4.0),
            WeaponTypes::Trap => (2.0, 4.0),
            WeaponTypes::LaserSword => (3.0, 15.0),
            WeaponTypes::Flame => (6.0, 4.0),
        };

        WeaponFire {
            width,
            height,
            dx: 0.0,
            dy: 0.0,
            spawn_x: 0.0,
            spawn_y: 0.0,
            spawn_angle,
            owner_player_id,
            damage,
            shot_speed,
            shot_life_timer: 0.0,
            shot_life_limit,
            shield_damage_pct,
            armor_damage_pct,
            piercing_damage_pct,
            health_damage_pct,
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

pub fn build_named_weapon(weapon_name: WeaponNames) -> WeaponStats {
    let weapon_configs_map: HashMap<WeaponNames, WeaponStats> = build_weapon_store();

    match weapon_configs_map.get(&weapon_name) {
        Some(weapon_config) => *weapon_config,
        _ => WeaponStats {
            weapon_type: WeaponTypes::LaserDouble,
            heat_seeking: false,
            heat_seeking_agility: 0.0,
            attached: false,
            deployed: false,
            tracking_angle: 0.0,
            spread_angle: 0.0,
            mounted_angle: 0.0,
            cooldown_reset: 100.0,
            burst_shot_limit: 0,
            burst_cooldown_reset: 100.0,
            shot_life_limit: 0.1,
            damage: 0.0,
            shot_speed: 0.0,
            shield_damage_pct: 0.0,
            armor_damage_pct: 0.0,
            piercing_damage_pct: 0.0,
            health_damage_pct: 0.0,
            weight: 0.0,
        },
    }
}

pub fn update_weapon_icon(
    entities: &Entities,
    weapon: &mut Weapon,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    player_id: usize,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let weapon_type = weapon.stats.weapon_type;

    //UI icon
    let weapon_entity: Entity = entities.create();

    weapon.icon_entity = weapon_entity;

    let x = 15.;
    let y = UI_HEIGHT - 10.;
    let dx = 32.;
    let dx2 = 4.;

    let weapon_icon_dx = 70.0;

    let (icon_scale, weapon_sprite) =
        get_weapon_icon(player_id, weapon.stats, weapon_fire_resource);

    let mut icon_weapon_transform = Transform::default();

    let starting_x = match player_id {
        0 => (x),
        1 => (x + 3.0 * dx + dx2),
        2 => (x + 6.0 * dx + 2.0 * dx2),
        3 => (x + 9.0 * dx + 3.0 * dx2),
        _ => (0.0),
    };

    icon_weapon_transform.set_translation_xyz((starting_x + weapon_icon_dx) as f32, y, 0.4);
    icon_weapon_transform.set_rotation_2d(-PI / 2.0);
    icon_weapon_transform.set_scale(Vector3::new(icon_scale, icon_scale, 0.0));

    let icon_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

    lazy_update.insert(weapon_entity, PlayerWeaponIcon::new(player_id, weapon_type));
    lazy_update.insert(weapon_entity, weapon_sprite);
    lazy_update.insert(weapon_entity, icon_weapon_transform);
    lazy_update.insert(weapon_entity, icon_tint);
    lazy_update.insert(weapon_entity, Transparent);
    lazy_update.insert(weapon_entity, Removal::new(0 as u32));
}

pub fn get_weapon_icon(
    player_id: usize,
    weapon_stats: WeaponStats,
    weapon_fire_resource: &WeaponFireResource,
) -> (f32, SpriteRender) {
    let (icon_scale, mut weapon_sprite) = match weapon_stats.weapon_type {
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
        WeaponTypes::Trap => (3.0, weapon_fire_resource.trap_p1_sprite_render.clone()),
        WeaponTypes::LaserSword => (1.0, weapon_fire_resource.laser_sword_sprite_render.clone()),
        WeaponTypes::Flame => (2.0, weapon_fire_resource.flame_sprite_render.clone()),
    };

    if weapon_stats.weapon_type == WeaponTypes::Mine {
        weapon_sprite = get_mine_sprite(player_id, weapon_stats.shot_speed, weapon_fire_resource);
    } else if weapon_stats.weapon_type == WeaponTypes::Trap {
        weapon_sprite = get_trap_sprite(player_id, weapon_fire_resource);
    }

    (icon_scale, weapon_sprite)
}

pub fn get_mine_sprite(
    player_id: usize,
    shot_speed: f32,
    weapon_fire_resource: &WeaponFireResource,
) -> SpriteRender {
    if shot_speed > 0.0 {
        weapon_fire_resource.mine_neutral_sprite_render.clone()
    } else {
        match player_id {
            0 => weapon_fire_resource.mine_p1_sprite_render.clone(),
            1 => weapon_fire_resource.mine_p2_sprite_render.clone(),
            2 => weapon_fire_resource.mine_p3_sprite_render.clone(),
            3 => weapon_fire_resource.mine_p4_sprite_render.clone(),
            _ => weapon_fire_resource.mine_neutral_sprite_render.clone(),
        }
    }
}

pub fn get_trap_sprite(
    player_id: usize,
    weapon_fire_resource: &WeaponFireResource,
) -> SpriteRender {
    match player_id {
        0 => weapon_fire_resource.trap_p1_sprite_render.clone(),
        1 => weapon_fire_resource.trap_p2_sprite_render.clone(),
        2 => weapon_fire_resource.trap_p3_sprite_render.clone(),
        3 => weapon_fire_resource.trap_p4_sprite_render.clone(),
        _ => weapon_fire_resource.trap_p1_sprite_render.clone(),
    }
}
