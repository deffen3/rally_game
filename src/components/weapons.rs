use amethyst::{
    assets::{Loader},
    core::math::Vector3,
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entities, Entity, LazyUpdate, ReadExpect, World},
    renderer::{palette::Srgba, resources::Tint, SpriteRender, Transparent},
    utils::{removal::Removal},
};

use rand::Rng;
use ron::de::from_reader;
use serde::Deserialize;
use std::f32::consts::PI;
use std::{collections::HashMap, fs::File};

use log::{info};

use crate::components::PlayerWeaponIcon;
use crate::rally::UI_HEIGHT;
use crate::resources::{GameWeaponSetup, WeaponFireResource};

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
    SmartRocketGrenade,
    Flamethrower,
    IonCannon,
}

pub fn get_random_weapon_name(game_setup: &ReadExpect<GameWeaponSetup>) -> WeaponNames {
    let mut rng = rand::thread_rng();
    let chance_selector = rng.gen_range(0.0, 1.0);

    let mut weapon_selector = game_setup.starter_weapon.clone();

    for (weapon_name, chance) in game_setup.weapon_spawn_chances.iter() {
        if *chance >= chance_selector {
            weapon_selector = weapon_name.clone();
            break;
        }
    }

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
        WeaponNames::LaserDoubleBurstSide => Some(WeaponNames::Mine),
        WeaponNames::Mine => Some(WeaponNames::LaserSword),
        WeaponNames::LaserSword => Some(WeaponNames::BackwardsLaserSword),
        WeaponNames::BackwardsLaserSword => None,
        WeaponNames::SmartRocketGrenade => None,
        WeaponNames::Trap => None,
        WeaponNames::ProjectileSteadyFire => None,
        WeaponNames::LaserDouble => None,
        WeaponNames::ProjectileRapidFire => None,
        WeaponNames::LaserPulse => None,
        WeaponNames::IonCannon => None,
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
    pub trigger_radius: f32,
    pub damage_radius: f32,
    pub shot_speed: f32,
    pub accel_rate: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
    pub ion_malfunction_pct: f32,
    pub weight: f32,
}

#[derive(Clone)]
pub struct WeaponStoreResource {
    pub store: HashMap<WeaponNames, WeaponStats>,
}

pub fn build_weapon_store(world: &mut World) -> WeaponStoreResource {
    // let loader = world.read_resource::<Loader>();
    // let weapon_configs_map = loader.load(
    //     "game/weapons.ron",
    //     RonFormat,
    //     (),
    // );

    let input_path = format!("{}/assets/game/weapons.ron", env!("CARGO_MANIFEST_DIR"));
    let f = File::open(&input_path).expect("Failed opening file");

    let weapon_configs_map: HashMap<WeaponNames, WeaponStats> =
        from_reader(f).expect("Failed to load config");

    let weapon_store = WeaponStoreResource {
        store: weapon_configs_map,
    };
    world.insert(weapon_store.clone());

    weapon_store
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
    Grenade,
    Ion,
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
    pub dps_calc: f32,
    pub range_calc: f32,
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
            dps_calc: calculate_dps(stats),
            range_calc: calculate_range(stats),
        }
    }
}



#[derive(Clone)]
pub struct WeaponArray {
    pub weapons: [Option<Weapon>; 4],
}

impl Component for WeaponArray {
    type Storage = DenseVecStorage<Self>;
}




pub fn calculate_dps(stats: WeaponStats) -> f32 {
    let dps;

    if stats.burst_shot_limit > 0 {
        dps = (stats.damage * ((stats.burst_shot_limit+1) as f32)) /
            (stats.cooldown_reset + ((stats.burst_shot_limit+1) as f32) *stats.burst_cooldown_reset);
    }
    else {
        dps = stats.damage / stats.cooldown_reset
    }

    dps
}

pub fn calculate_range(stats: WeaponStats) -> f32 {
    let range;
    
    if stats.shot_speed <= 0.0 {
        range = 0.0;
    }
    else if stats.shot_life_limit <= 0.0 {
        range = 10000.0;
    }
    else {
        range = stats.shot_speed * stats.shot_life_limit;
    }
    
    range
}



#[derive(Clone)]
pub struct WeaponFire {
    pub active: bool,
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub spawn_angle: f32,
    pub owner_player_id: usize,
    pub shot_speed: f32,
    pub accel_rate: f32,
    pub shot_life_timer: f32,
    pub shot_life_limit: f32,
    pub damage: f32,
    pub trigger_radius: f32,
    pub damage_radius: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
    pub ion_malfunction_pct: f32,
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
        accel_rate: f32,
        shot_life_limit: f32,
        damage: f32,
        trigger_radius: f32,
        damage_radius: f32,
        shield_damage_pct: f32,
        armor_damage_pct: f32,
        piercing_damage_pct: f32,
        health_damage_pct: f32,
        ion_malfunction_pct: f32,
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
            WeaponTypes::Grenade => (4.0, 4.0),
            WeaponTypes::Trap => (2.0, 4.0),
            WeaponTypes::LaserSword => (3.0, 15.0),
            WeaponTypes::Flame => (6.0, 4.0),
            WeaponTypes::Ion => (5.0, 5.0),
        };

        WeaponFire {
            active: true,
            width,
            height,
            dx: 0.0,
            dy: 0.0,
            spawn_x: 0.0,
            spawn_y: 0.0,
            spawn_angle,
            owner_player_id,
            damage,
            trigger_radius,
            damage_radius,
            shot_speed,
            accel_rate,
            shot_life_timer: 0.0,
            shot_life_limit,
            shield_damage_pct,
            armor_damage_pct,
            piercing_damage_pct,
            health_damage_pct,
            ion_malfunction_pct,
            heat_seeking,
            heat_seeking_agility,
            attached,
            deployed,
            weapon_name,
            weapon_type,
        }
    }
}

pub fn update_weapon_properties(
    weapon_array: &mut WeaponArray,
    weapon_name: WeaponNames,
    weapon_store: &ReadExpect<WeaponStoreResource>,
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    player_id: usize,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    //Get new weapon data
    let new_weapon_stats = build_named_weapon(weapon_name.clone(), weapon_store);
    let weapon_type = new_weapon_stats.weapon_type;

    //update UI icon
    let icon_entity: Entity = entities.create();

    let x = 15.;
    let y = UI_HEIGHT - 10.;
    let dx = 32.;
    let dx2 = 4.;

    let weapon_icon_dx = 70.0;

    let (icon_scale, weapon_sprite) =
        get_weapon_icon(player_id, new_weapon_stats, weapon_fire_resource);

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

    lazy_update.insert(icon_entity, PlayerWeaponIcon::new(player_id, weapon_type));
    lazy_update.insert(icon_entity, weapon_sprite);
    lazy_update.insert(icon_entity, icon_weapon_transform);
    lazy_update.insert(icon_entity, icon_tint);
    lazy_update.insert(icon_entity, Transparent);
    lazy_update.insert(icon_entity, Removal::new(0 as u32));


    //update Weapon
    let new_primary_weapon = Weapon::new(weapon_name, icon_entity, new_weapon_stats);

    info!("{:?} {:?} {:?}", new_primary_weapon.name, new_primary_weapon.dps_calc, new_primary_weapon.range_calc);

    weapon_array.weapons[0] = Some(new_primary_weapon);
}

pub fn build_named_weapon(
    weapon_name: WeaponNames,
    weapon_store: &ReadExpect<WeaponStoreResource>,
) -> WeaponStats {
    let weapon_configs_map: &HashMap<WeaponNames, WeaponStats> = &weapon_store.store;

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
            trigger_radius: 0.0,
            damage_radius: 0.0,
            shot_speed: 0.0,
            accel_rate: 0.0,
            shield_damage_pct: 0.0,
            armor_damage_pct: 0.0,
            piercing_damage_pct: 0.0,
            health_damage_pct: 0.0,
            ion_malfunction_pct: 0.0,
            weight: 0.0,
        },
    }
}

pub fn build_named_weapon2(
    weapon_name: WeaponNames,
    weapon_store: WeaponStoreResource,
) -> WeaponStats {
    let weapon_configs_map: HashMap<WeaponNames, WeaponStats> = weapon_store.store;

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
            trigger_radius: 0.0,
            damage_radius: 0.0,
            shot_speed: 0.0,
            accel_rate: 0.0,
            shield_damage_pct: 0.0,
            armor_damage_pct: 0.0,
            piercing_damage_pct: 0.0,
            health_damage_pct: 0.0,
            ion_malfunction_pct: 0.0,
            weight: 0.0,
        },
    }
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
        WeaponTypes::Grenade => (2.0, weapon_fire_resource.grenade_sprite_render.clone()),
        WeaponTypes::Ion => (2.0, weapon_fire_resource.ion_sprite_render.clone()),
    };

    //Player colored weapons
    if weapon_stats.weapon_type == WeaponTypes::Mine {
        weapon_sprite = get_mine_sprite(player_id, weapon_fire_resource);
    } else if weapon_stats.weapon_type == WeaponTypes::Trap {
        weapon_sprite = get_trap_sprite(player_id, weapon_fire_resource);
    }

    (icon_scale, weapon_sprite)
}

pub fn get_mine_sprite(
    player_id: usize,
    weapon_fire_resource: &WeaponFireResource,
) -> SpriteRender {
    match player_id {
        0 => weapon_fire_resource.mine_p1_sprite_render.clone(),
        1 => weapon_fire_resource.mine_p2_sprite_render.clone(),
        2 => weapon_fire_resource.mine_p3_sprite_render.clone(),
        3 => weapon_fire_resource.mine_p4_sprite_render.clone(),
        _ => weapon_fire_resource.mine_p1_sprite_render.clone(),
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
