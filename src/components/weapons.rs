use amethyst::{
    ecs::prelude::{Component, DenseVecStorage, Entities, Entity, LazyUpdate, ReadExpect, World},
    renderer::{palette::Srgba, resources::Tint, SpriteRender, Transparent},
    ui::{Anchor, UiImage, UiTransform},
    utils::removal::Removal,
};

use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;

use log::info;

use crate::components::PlayerWeaponIcon;
use crate::load_ron_asset;
use crate::resources::{GameWeaponSelectionMode, GameWeaponSetup, WeaponFireResource};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Hash, Eq)]
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
    PiercingProjectile,
    ProjectileCannon,
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
    BioSpiker,
    StormGun,
    SlimeLauncher,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum WeaponFireTypes {
    LaserBeam,
    LaserPulse,
    LaserDouble,
    ProjectileSmall,
    ProjectileMedium,
    ProjectileLarge,
    Mine,
    Trap,
    Missile,
    Rockets,
    LaserSword,
    Flame,
    Grenade,
    Ion,
    BioSpike,
    LightBolt,
    SlimeBall,
}

//For when picking up weapon spawn boxes, or other random weapon selections
//This one is built to use pre-made chance lists
pub fn get_random_weapon_name(
    random_weapon_spawn_chances: &Vec<(WeaponNames, f32)>,
) -> Option<WeaponNames> {
    let mut rng = rand::thread_rng();
    let chance_selector = rng.gen_range(0.0, 1.0);

    let mut weapon_selector = None;

    for (weapon_name, chance) in random_weapon_spawn_chances.iter() {
        if *chance >= chance_selector {
            break; //stay on previously selected weapon
        } else {
            weapon_selector = Some(weapon_name.clone()); //keep setting selected weapon, until break
        }
    }

    weapon_selector
}

//This one can build the chance list on the fly from the relative chance list
pub fn get_random_weapon_name_build_chance(
    random_weapon_spawn_relative_chance: &Option<Vec<(WeaponNames, u32)>>,
) -> Option<WeaponNames> {
    let mut weapon_selector = None;

    if let Some(random_weapon_spawn_relative_chance) = random_weapon_spawn_relative_chance {
        let mut rng = rand::thread_rng();
        let chance_selector = rng.gen_range(0.0, 1.0);

        let mut chance_total: u32 = 0;

        for (_weapon_name, value) in random_weapon_spawn_relative_chance.iter() {
            chance_total += value;
        }

        let mut chance_aggregate: f32 = 0.0;
        let mut weapon_spawn_chances = vec![];

        for (weapon_name, value) in random_weapon_spawn_relative_chance.iter() {
            if *value > 0 {
                weapon_spawn_chances.push((weapon_name.clone(), chance_aggregate));

                chance_aggregate += (*value as f32) / (chance_total as f32);
            }
        }

        for (weapon_name, chance) in weapon_spawn_chances.iter() {
            if *chance >= chance_selector {
                break; //stay on previously selected weapon
            } else {
                weapon_selector = Some(weapon_name.clone()); //keep setting selected weapon, until break
            }
        }
    }

    weapon_selector
}

pub fn get_next_gg_weapon_name(
    weapon_name: Option<WeaponNames>,
    weapon_store_resource: &WeaponStoreResource,
    game_weapon_setup: &GameWeaponSetup,
) -> Option<WeaponNames> {
    //if weapon_name is None, start at beginning of the list

    let length = weapon_store_resource.gun_game_order.len();

    let weapon_out: Option<WeaponNames>;
    if game_weapon_setup.mode == GameWeaponSelectionMode::GunGameForward {
        if let Some(weapon_name) = weapon_name {
            let index = weapon_store_resource
                .gun_game_order
                .iter()
                .position(|&r| r == weapon_name);
            if let Some(index) = index {
                if index == length - 1 {
                    weapon_out = Some(weapon_store_resource.gun_game_order[0]); //loop-back around
                } else {
                    weapon_out = Some(weapon_store_resource.gun_game_order[index + 1]);
                }
            } else {
                weapon_out = None;
            }
        } else {
            weapon_out = Some(weapon_store_resource.gun_game_order[0]); //start at beginning
        }
    } else if game_weapon_setup.mode == GameWeaponSelectionMode::GunGameReverse {
        if let Some(weapon_name) = weapon_name {
            let index = weapon_store_resource
                .gun_game_order
                .iter()
                .position(|&r| r == weapon_name);
            if let Some(index) = index {
                if index == 0 {
                    weapon_out = Some(weapon_store_resource.gun_game_order[length - 1]);
                //loop-back around
                } else {
                    weapon_out = Some(weapon_store_resource.gun_game_order[index - 1]);
                }
            } else {
                weapon_out = None;
            }
        } else {
            weapon_out = Some(weapon_store_resource.gun_game_order[length - 1]);
            //start at end, which is beginning in this mode
        }
    } else if game_weapon_setup.mode == GameWeaponSelectionMode::GunGameRandom {
        if let Some(weapon_name) = weapon_name {
            let index = weapon_store_resource
                .gun_game_random_order
                .iter()
                .position(|&r| r == weapon_name);
            if let Some(index) = index {
                if index == length - 1 {
                    weapon_out = Some(weapon_store_resource.gun_game_random_order[0]);
                //loop-back around
                } else {
                    weapon_out = Some(weapon_store_resource.gun_game_random_order[index + 1]);
                }
            } else {
                weapon_out = None;
            }
        } else {
            weapon_out = Some(weapon_store_resource.gun_game_random_order[0]); //start at beginning
        }
    } else {
        weapon_out = None;
    }
    weapon_out
}

#[derive(Copy, Clone, Debug, Deserialize, Default)]
pub struct DurationDamage {
    pub timer: f32,
    pub damage_per_second: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
    pub ion_malfunction_pct: f32,
}

#[derive(Copy, Clone, Debug, Deserialize, Default)]
pub struct ChainingDamage {
    pub damage_pct: f32,
    pub radius: f32,
    pub jumps: u32,
    pub prongs: u32,
}

#[derive(Copy, Clone, Debug, Deserialize, Default)]
pub struct SlowDownEffect {
    pub timer: f32,
    pub slow_down_pct: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WeaponStats {
    pub display_name: String,
    pub weapon_fire_type: WeaponFireTypes,
    pub tracking_angle: f32,
    pub tracking_range: f32,
    pub spread_angle: f32,
    pub cooldown_reset: f32,
    pub burst_shot_limit: u32,
    pub burst_cooldown_reset: f32,
    pub charge_timer_reset: f32,
    pub charge_timer_decrease: f32,
    pub charge_timer_decrease_min: f32,
    pub spin_up_timer_reset: f32,
    pub weight: f32,
    pub fire_stats: WeaponFireStats,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct WeaponFireStats {
    pub heat_seeking: bool,
    pub heat_seeking_agility: f32,
    pub attached: bool,
    pub mount_angle_special_offset: f32,
    pub shot_life_limit: f32,
    pub damage: f32,
    pub damage_reduction_pct_rate: f32,
    pub trigger_radius: f32,
    pub trigger_immediately: bool,
    pub damage_radius: f32,
    pub shot_speed: f32,
    pub accel_rate: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
    pub ion_malfunction_pct: f32,
    pub duration_damage: DurationDamage,
    pub bounces: u32,
    pub chaining_damage: ChainingDamage,
    pub slow_down_effect: SlowDownEffect,
    pub stuck_accel_effect_timer: f32,
}

#[derive(Clone)]
pub struct WeaponStoreResource {
    pub properties: HashMap<WeaponNames, WeaponStats>,
    pub spawn_chance: HashMap<WeaponNames, u32>,
    pub gun_game_order: Vec<WeaponNames>,
    pub gun_game_random_order: Vec<WeaponNames>,
    pub selection_order: Vec<WeaponNames>,
}

pub fn build_weapon_store(world: &mut World) {
    world.insert(WeaponStoreResource {
        properties: load_ron_asset(&["game", "weapon_properties.ron"]),
        spawn_chance: load_ron_asset(&["game", "weapon_spawn_chance.ron"]),
        gun_game_order: load_ron_asset(&["game", "weapon_gun_game_order.ron"]),
        gun_game_random_order: load_ron_asset(&["game", "weapon_gun_game_order.ron"]),
        selection_order: load_ron_asset(&["game", "weapon_selection_order.ron"]),
    });
}

#[derive(Clone)]
pub struct Weapon {
    pub name: WeaponNames,
    pub icon_entity: Entity,
    pub x: f32,
    pub y: f32,
    pub stats: WeaponStats,
    pub cooldown_timer: f32,
    pub charge_timer: f32,
    pub charges: u32,
    pub spin_up_timer: f32,
    pub burst_shots: u32,
    pub dps_calc: f32,
    pub range_calc: f32,
    pub deployed: bool,
    pub deploy_timer: f32,
    pub ammo: Option<u32>,
}

impl Weapon {
    pub fn new(
        name: WeaponNames,
        icon_entity: Entity,
        stats: WeaponStats,
        ammo: Option<u32>,
    ) -> Weapon {
        Weapon {
            name,
            icon_entity,
            x: 0.0,
            y: 0.0,
            stats: stats.clone(),
            cooldown_timer: 0.0,
            charge_timer: stats.charge_timer_reset,
            charges: 0,
            spin_up_timer: stats.spin_up_timer_reset,
            burst_shots: 0,
            dps_calc: calculate_dps(stats.clone()),
            range_calc: calculate_range(stats.fire_stats),
            deployed: false,
            deploy_timer: 0.0,
            ammo: ammo,
        }
    }
}

#[derive(Clone)]
pub struct WeaponInstall {
    pub weapon: Weapon,
    pub firing_group: u8,
    pub ammo: Option<u32>,
    pub mounted_angle: Option<f32>,
    pub x_offset: Option<f32>,
    pub y_offset: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct WeaponNameInstall {
    pub weapon_name: WeaponNames,
    pub firing_group: u8,
    pub ammo: Option<u32>,
    pub mounted_angle: Option<f32>,
    pub x_offset: Option<f32>,
    pub y_offset: Option<f32>,
}

#[derive(Clone)]
pub struct WeaponArray {
    pub installed: Vec<WeaponInstall>,
}

impl Component for WeaponArray {
    type Storage = DenseVecStorage<Self>;
}

pub fn calculate_dps(stats: WeaponStats) -> f32 {
    let base_damage: f32 = stats.fire_stats.damage; //damage per shot
    let fire_rate: f32; //shots per second

    if stats.burst_shot_limit > 0 {
        fire_rate = ((stats.burst_shot_limit + 1) as f32)
            / (stats.cooldown_reset
                + ((stats.burst_shot_limit + 1) as f32) * stats.burst_cooldown_reset);
    } else {
        fire_rate = 1.0 / stats.cooldown_reset
    }

    //let duration_damage: f32 = stats.duration_damage.damage_per_second * stats.duration_damage.timer;

    let dps = base_damage * fire_rate;

    dps
}

pub fn calculate_range(stats: WeaponFireStats) -> f32 {
    let range;
    if stats.shot_speed <= 0.0 {
        range = 0.0;
    } else if stats.shot_life_limit <= 0.0 {
        range = 10000.0;
    } else {
        range = stats.shot_speed * stats.shot_life_limit;
    }
    range
}

#[derive(Clone, Debug)]
pub struct WeaponFire {
    pub weapon_array_id: usize,
    pub weapon_fire_type: WeaponFireTypes,
    pub weapon_name: WeaponNames,
    pub active: bool,
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
    pub weapon_angle_offset: f32,
    pub owner_player_id: Option<usize>,
    pub shot_life_timer: f32,
    pub chain_hit_ids: Vec<usize>,
    pub stats: WeaponFireStats,
    pub deployed: bool,
}

impl Component for WeaponFire {
    type Storage = DenseVecStorage<Self>;
}

impl WeaponFire {
    pub fn new(
        owner_player_id: Option<usize>,
        weapon_array_id: usize,
        weapon_name: WeaponNames,
        weapon_fire_type: WeaponFireTypes,
        weapon_angle_offset: f32,
        stats: WeaponFireStats,
    ) -> WeaponFire {
        let (width, height) = get_weapon_width_height(weapon_fire_type.clone());

        WeaponFire {
            weapon_array_id,
            weapon_name,
            weapon_fire_type,
            active: true,
            width,
            height,
            dx: 0.0,
            dy: 0.0,
            weapon_angle_offset,
            owner_player_id,
            shot_life_timer: 0.0,
            chain_hit_ids: Vec::<usize>::new(),
            stats,
            deployed: false,
        }
    }
}

pub fn update_weapon_properties(
    weapon_array: &mut WeaponArray,
    weapon_array_id: usize,
    firing_group: u8,
    ammo: Option<u32>,
    weapon_name: Option<WeaponNames>,
    weapon_store: &ReadExpect<WeaponStoreResource>,
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    player_id: Option<usize>,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    if let Some(weapon_name) = weapon_name {
        let player_id_sub;
        if player_id.is_none() {
            player_id_sub = 0;
        } else {
            player_id_sub = player_id.unwrap();
        }

        //Get new weapon data
        let new_weapon_stats = build_named_weapon(weapon_name.clone(), weapon_store);
        let weapon_fire_type = new_weapon_stats.weapon_fire_type;

        //update UI icon
        let icon_entity: Entity = entities.create();

        let x = -320. + (weapon_array_id as f32) * 30.0;
        let y = 45.;
        let dx = 250.;

        let (icon_scale, weapon_sprite) = get_weapon_icon(
            player_id,
            new_weapon_stats.weapon_fire_type,
            &weapon_fire_resource,
        );

        let starting_x = match player_id_sub {
            0 => (x),
            1 => (x + dx),
            2 => (x + 2.0 * dx),
            3 => (x + 3.0 * dx),
            _ => (0.0),
        };

        let (width, height) = get_weapon_width_height(weapon_fire_type.clone());

        let icon_weapon_transform = UiTransform::new(
            "P1_WeaponIcon".to_string(),
            Anchor::BottomMiddle,
            Anchor::BottomMiddle,
            starting_x,
            y,
            0.2,
            width * icon_scale,
            height * icon_scale,
        );

        // White shows the sprite as normal.
        // You can change the color at any point to modify the sprite's tint.
        let icon_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

        lazy_update.insert(
            icon_entity,
            PlayerWeaponIcon::new(player_id_sub, weapon_array_id, weapon_fire_type),
        );
        lazy_update.insert(icon_entity, UiImage::Sprite(weapon_sprite));
        lazy_update.insert(icon_entity, icon_weapon_transform);
        lazy_update.insert(icon_entity, icon_tint);
        lazy_update.insert(icon_entity, Transparent);
        lazy_update.insert(icon_entity, Removal::new(0 as u32));

        //update Weapon
        let new_weapon = Weapon::new(weapon_name, icon_entity, new_weapon_stats, ammo);

        info!(
            "{:?} {:?} {:?}",
            new_weapon.name, new_weapon.dps_calc, new_weapon.range_calc
        );

        if weapon_array_id >= weapon_array.installed.len() {
            weapon_array.installed.push(WeaponInstall {
                weapon: new_weapon,
                firing_group,
                ammo,
                mounted_angle: None,
                x_offset: None,
                y_offset: None,
            });
        } else {
            weapon_array.installed[weapon_array_id] = WeaponInstall {
                weapon: new_weapon,
                firing_group,
                ammo,
                mounted_angle: None,
                x_offset: None,
                y_offset: None,
            };
        }
    } else {
        if weapon_array_id >= weapon_array.installed.len() { //weapon already removed
        } else {
            weapon_array.installed.remove(weapon_array_id); //remove weapon at this index
        }
    }
}

pub fn build_named_weapon(
    weapon_name: WeaponNames,
    weapon_store: &WeaponStoreResource,
) -> WeaponStats {
    let weapon_configs_map: &HashMap<WeaponNames, WeaponStats> = &weapon_store.properties;

    match weapon_configs_map.get(&weapon_name) {
        Some(weapon_config) => (*weapon_config).clone(),
        _ => WeaponStats {
            weapon_fire_type: WeaponFireTypes::LaserDouble,
            display_name: "None".to_string(),
            tracking_angle: 0.0,
            tracking_range: 0.0,
            spread_angle: 0.0,
            cooldown_reset: 0.0,
            charge_timer_reset: 0.0,
            charge_timer_decrease: 0.0,
            charge_timer_decrease_min: 0.0,
            spin_up_timer_reset: 0.0,
            burst_shot_limit: 0,
            burst_cooldown_reset: 0.0,
            weight: 0.0,
            fire_stats: WeaponFireStats::default(),
        },
    }
}

pub fn build_named_weapon_from_world(weapon_name: WeaponNames, world: &mut World) -> WeaponStats {
    let weapon_store = world.fetch::<WeaponStoreResource>();
    let weapon_stats = build_named_weapon(weapon_name, &weapon_store);

    weapon_stats
}

pub fn get_weapon_width_height(weapon_fire_type: WeaponFireTypes) -> (f32, f32) {
    let (width, height) = match weapon_fire_type {
        WeaponFireTypes::LaserDouble => (3.0, 6.0),
        WeaponFireTypes::LaserBeam => (1.0, 12.0),
        WeaponFireTypes::LaserPulse => (2.0, 5.0),
        WeaponFireTypes::ProjectileMedium => (1.0, 4.0),
        WeaponFireTypes::ProjectileSmall => (1.0, 2.0),
        WeaponFireTypes::ProjectileLarge => (3.0, 3.0),
        WeaponFireTypes::Missile => (5.0, 6.0),
        WeaponFireTypes::Rockets => (5.0, 4.0),
        WeaponFireTypes::Mine => (4.0, 4.0),
        WeaponFireTypes::Grenade => (4.0, 4.0),
        WeaponFireTypes::Trap => (2.0, 4.0),
        WeaponFireTypes::LaserSword => (3.0, 15.0),
        WeaponFireTypes::Flame => (6.0, 4.0),
        WeaponFireTypes::Ion => (5.0, 5.0),
        WeaponFireTypes::BioSpike => (3.0, 9.0),
        WeaponFireTypes::LightBolt => (3.0, 7.0),
        WeaponFireTypes::SlimeBall => (6.0, 5.0),
    };

    (width, height)
}

pub fn get_weapon_icon(
    player_id: Option<usize>,
    weapon_fire_type: WeaponFireTypes,
    weapon_fire_resource: &WeaponFireResource,
) -> (f32, SpriteRender) {
    let player_id_sub;
    if player_id.is_none() {
        player_id_sub = 0;
    } else {
        player_id_sub = player_id.unwrap();
    }

    let (icon_scale, mut weapon_sprite) = match weapon_fire_type {
        WeaponFireTypes::LaserDouble => {
            (1.5, weapon_fire_resource.laser_double_sprite_render.clone())
        }
        WeaponFireTypes::LaserBeam => (0.5, weapon_fire_resource.laser_beam_sprite_render.clone()),
        WeaponFireTypes::LaserPulse => {
            (1.5, weapon_fire_resource.laser_burst_sprite_render.clone())
        }
        WeaponFireTypes::ProjectileMedium => {
            (1.5, weapon_fire_resource.projectile_burst_render.clone())
        }
        WeaponFireTypes::ProjectileSmall => {
            (1.5, weapon_fire_resource.projectile_rapid_render.clone())
        }
        WeaponFireTypes::ProjectileLarge => (
            1.5,
            weapon_fire_resource.projectile_cannon_sprite_render.clone(),
        ),
        WeaponFireTypes::Missile => (1.0, weapon_fire_resource.missile_sprite_render.clone()),
        WeaponFireTypes::Rockets => (1.0, weapon_fire_resource.rockets_sprite_render.clone()),
        WeaponFireTypes::Mine => (1.0, weapon_fire_resource.mine_p1_sprite_render.clone()),
        WeaponFireTypes::Trap => (1.5, weapon_fire_resource.trap_p1_sprite_render.clone()),
        WeaponFireTypes::LaserSword => {
            (0.5, weapon_fire_resource.laser_sword_sprite_render.clone())
        }
        WeaponFireTypes::Flame => (1.0, weapon_fire_resource.flame_sprite_render.clone()),
        WeaponFireTypes::Grenade => (1.0, weapon_fire_resource.grenade_sprite_render.clone()),
        WeaponFireTypes::Ion => (1.0, weapon_fire_resource.ion_sprite_render.clone()),
        WeaponFireTypes::BioSpike => (0.75, weapon_fire_resource.bio_spike_sprite_render.clone()),
        WeaponFireTypes::LightBolt => (1.0, weapon_fire_resource.light_bolt_sprite_render.clone()),
        WeaponFireTypes::SlimeBall => (1.0, weapon_fire_resource.slime_ball_sprite_render.clone()),
    };

    //Player colored weapons
    if weapon_fire_type == WeaponFireTypes::Mine {
        weapon_sprite = get_mine_sprite(player_id_sub, weapon_fire_resource);
    } else if weapon_fire_type == WeaponFireTypes::Trap {
        weapon_sprite = get_trap_sprite(player_id_sub, weapon_fire_resource);
    }

    (icon_scale * 3.0, weapon_sprite)
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
