use amethyst::{
    ecs::prelude::{Component, DenseVecStorage, Entities, Entity, LazyUpdate, ReadExpect, World},
    renderer::{palette::Srgba, resources::Tint, SpriteRender, Transparent},
    utils::{removal::Removal, application_root_dir},
    ui::{UiTransform, UiImage, Anchor},
};

use rand::Rng;
use ron::de::from_reader;
use serde::Deserialize;
use std::{collections::HashMap, fs::File};
use std::env::current_dir;

use log::{info};

use crate::components::PlayerWeaponIcon;
use crate::resources::{GameWeaponSetup, WeaponFireResource};


pub const WEAPON_ARRAY_SIZE: usize = 4;


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


//For when picking up weapon spawn boxes, or other random weapon selections
pub fn get_random_weapon_name(game_setup: &ReadExpect<GameWeaponSetup>) -> WeaponNames {
    let mut rng = rand::thread_rng();
    let chance_selector = rng.gen_range(0.0, 1.0);

    let mut weapon_selector = game_setup.starter_weapon.clone();

    for (weapon_name, chance) in game_setup.weapon_spawn_chances.iter() {
        if *chance >= chance_selector {
            break; //stay on previously selected weapon
        }
        else {
            weapon_selector = weapon_name.clone();
        }
    }

    weapon_selector
}



pub fn get_next_weapon_name(
    weapon_name: WeaponNames,
    weapon_store_resource: &WeaponStoreResource,
) -> Option<WeaponNames> {

    let length = weapon_store_resource.gun_game_order.len();
    let index = weapon_store_resource.gun_game_order.iter().position(|&r| r == weapon_name);

    let weapon_out: Option<WeaponNames>;
    if let Some(index) = index {
        if index == length-1 {
            weapon_out = Some(weapon_store_resource.gun_game_order[0]); //loop-back around
        }
        else {
            weapon_out = Some(weapon_store_resource.gun_game_order[index+1]);
        }
    }
    else {
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
    pub spread_angle: f32,
    pub cooldown_reset: f32,
    pub burst_shot_limit: u32,
    pub burst_cooldown_reset: f32,
    pub charge_timer_reset: f32,
    pub spin_up_timer_reset: f32,
    pub weight: f32,
    pub fire_stats: WeaponFireStats,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct WeaponFireStats {
    pub heat_seeking: bool,
    pub heat_seeking_agility: f32,
    pub attached: bool,
    pub deployed: bool,
    pub mounted_angle: f32,
    pub shot_life_limit: f32,
    pub damage: f32,
    pub damage_reduction_rate: f32,
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
    pub selection_order: Vec<WeaponNames>,
}

pub fn build_weapon_store(world: &mut World) {
    // let app_root = current_dir();
    // let input_path = app_root.unwrap().join("assets/game/weapons.ron");

    let input_path_weapon_props = format!("{}/assets/game/weapon_properties.ron", env!("CARGO_MANIFEST_DIR"));
    let input_path_spawn_chance = format!("{}/assets/game/weapon_spawn_chance.ron", env!("CARGO_MANIFEST_DIR"));
    let input_path_gun_game_order = format!("{}/assets/game/weapon_gun_game_order.ron", env!("CARGO_MANIFEST_DIR"));
    let input_path_selection_order = format!("{}/assets/game/weapon_selection_order.ron", env!("CARGO_MANIFEST_DIR"));

    let f_weapon_props = File::open(&input_path_weapon_props).expect("Failed opening file");
    let f_spawn_chance = File::open(&input_path_spawn_chance).expect("Failed opening file");
    let f_gun_game_order = File::open(&input_path_gun_game_order).expect("Failed opening file");
    let f_selection_order = File::open(&input_path_selection_order).expect("Failed opening file");

    let weapon_properties_map: HashMap<WeaponNames, WeaponStats> =
        from_reader(f_weapon_props).expect("Failed to load config");
    let weapon_spawn_chance_map: HashMap<WeaponNames, u32> =
        from_reader(f_spawn_chance).expect("Failed to load config");
    let gun_game_order_map: Vec<WeaponNames> =
        from_reader(f_gun_game_order).expect("Failed to load config");
    let selection_order_map: Vec<WeaponNames> =
        from_reader(f_selection_order).expect("Failed to load config");


    let weapon_store = WeaponStoreResource {
        properties: weapon_properties_map,
        spawn_chance: weapon_spawn_chance_map,
        gun_game_order: gun_game_order_map,
        selection_order: selection_order_map,
    };

    world.insert(weapon_store.clone());
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum WeaponFireTypes {
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
    pub charge_timer: f32,
    pub spin_up_timer: f32,
    pub burst_shots: u32,
    pub dps_calc: f32,
    pub range_calc: f32,
    pub deploy_timer: f32,
}

impl Weapon {
    pub fn new(name: WeaponNames, icon_entity: Entity, stats: WeaponStats) -> Weapon {
        Weapon {
            name,
            icon_entity,
            x: 0.0,
            y: 0.0,
            aim_angle: 0.0,
            stats: stats.clone(),
            cooldown_timer: 0.0,
            charge_timer: stats.charge_timer_reset,
            spin_up_timer: stats.spin_up_timer_reset,
            burst_shots: 0,
            dps_calc: calculate_dps(stats.clone()),
            range_calc: calculate_range(stats.fire_stats),
            deploy_timer: 0.0,
        }
    }
}



#[derive(Clone)]
pub struct WeaponArray {
    pub weapons: [Option<Weapon>; WEAPON_ARRAY_SIZE],
}

impl Component for WeaponArray {
    type Storage = DenseVecStorage<Self>;
}




pub fn calculate_dps(stats: WeaponStats) -> f32 {
    let base_damage: f32 = stats.fire_stats.damage; //damage per shot
    let fire_rate: f32; //shots per second

    if stats.burst_shot_limit > 0 {
        fire_rate = ((stats.burst_shot_limit+1) as f32) /
            (stats.cooldown_reset + ((stats.burst_shot_limit+1) as f32) *stats.burst_cooldown_reset);
    }
    else {
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
    }
    else if stats.shot_life_limit <= 0.0 {
        range = 10000.0;
    }
    else {
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
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub spawn_angle: f32,
    pub owner_player_id: usize,
    pub shot_life_timer: f32,
    pub chain_hit_ids: Vec<usize>,
    pub stats: WeaponFireStats,
}

impl Component for WeaponFire {
    type Storage = DenseVecStorage<Self>;
}

impl WeaponFire {
    pub fn new(
        owner_player_id: usize,
        weapon_array_id: usize,
        weapon_name: WeaponNames,
        weapon_fire_type: WeaponFireTypes,
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
            spawn_x: 0.0,
            spawn_y: 0.0,
            spawn_angle: 0.0,
            owner_player_id,
            shot_life_timer: 0.0,
            chain_hit_ids: Vec::<usize>::new(),
            stats,  
        }
    }
}

pub fn update_weapon_properties(
    weapon_array: &mut WeaponArray,
    weapon_index: usize,
    weapon_name: WeaponNames,
    weapon_store: &ReadExpect<WeaponStoreResource>,
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    player_id: usize,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    //Get new weapon data
    let new_weapon_stats = build_named_weapon(weapon_name.clone(), weapon_store);
    let weapon_fire_type = new_weapon_stats.weapon_fire_type;

    //update UI icon
    let icon_entity: Entity = entities.create();

    let x = -290. + (weapon_index as f32)*10.0;
    let y = 45.;
    let dx = 250.;

    let (icon_scale, weapon_sprite) =
        get_weapon_icon(player_id, new_weapon_stats.weapon_fire_type, &weapon_fire_resource);


    let starting_x = match player_id {
        0 => (x),
        1 => (x + dx),
        2 => (x + 2.0*dx),
        3 => (x + 3.0*dx),
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

    lazy_update.insert(icon_entity, PlayerWeaponIcon::new(player_id, weapon_fire_type));
    lazy_update.insert(icon_entity, UiImage::Sprite(weapon_sprite));
    lazy_update.insert(icon_entity, icon_weapon_transform);
    lazy_update.insert(icon_entity, icon_tint);
    lazy_update.insert(icon_entity, Transparent);
    lazy_update.insert(icon_entity, Removal::new(0 as u32));


    //update Weapon
    let new_weapon = Weapon::new(weapon_name, icon_entity, new_weapon_stats);

    info!("{:?} {:?} {:?}", new_weapon.name, new_weapon.dps_calc, new_weapon.range_calc);

    if weapon_index >= WEAPON_ARRAY_SIZE {
        weapon_array.weapons[WEAPON_ARRAY_SIZE-1] = Some(new_weapon);
    }
    else {
        weapon_array.weapons[weapon_index] = Some(new_weapon);
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
            spread_angle: 0.0,
            cooldown_reset: 0.0,
            charge_timer_reset: 0.0,
            spin_up_timer_reset: 0.0,
            burst_shot_limit: 0,
            burst_cooldown_reset: 0.0,
            weight: 0.0,
            fire_stats: WeaponFireStats::default(),
        },
    }
}


pub fn build_named_weapon_from_world(
    weapon_name: WeaponNames,
    world: &mut World,
) -> WeaponStats {
    let weapon_store = world.fetch::<WeaponStoreResource>();
    
    let weapon_stats = build_named_weapon(weapon_name, &weapon_store);

    weapon_stats
}


pub fn get_weapon_icon(
    player_id: usize,
    weapon_fire_type: WeaponFireTypes,
    weapon_fire_resource: &WeaponFireResource,
) -> (f32, SpriteRender) {

    let (icon_scale, mut weapon_sprite) = match weapon_fire_type {
        WeaponFireTypes::LaserDouble => (1.5, weapon_fire_resource.laser_double_sprite_render.clone()),
        WeaponFireTypes::LaserBeam => (0.5, weapon_fire_resource.laser_beam_sprite_render.clone()),
        WeaponFireTypes::LaserPulse => (1.5, weapon_fire_resource.laser_burst_sprite_render.clone()),
        WeaponFireTypes::ProjectileBurstFire => {
            (1.5, weapon_fire_resource.projectile_burst_render.clone())
        }
        WeaponFireTypes::ProjectileRapidFire => {
            (1.5, weapon_fire_resource.projectile_rapid_render.clone())
        }
        WeaponFireTypes::ProjectileCannonFire => (
            1.5,
            weapon_fire_resource.projectile_cannon_sprite_render.clone(),
        ),
        WeaponFireTypes::Missile => (1.0, weapon_fire_resource.missile_sprite_render.clone()),
        WeaponFireTypes::Rockets => (1.0, weapon_fire_resource.rockets_sprite_render.clone()),
        WeaponFireTypes::Mine => (1.0, weapon_fire_resource.mine_p1_sprite_render.clone()),
        WeaponFireTypes::Trap => (1.5, weapon_fire_resource.trap_p1_sprite_render.clone()),
        WeaponFireTypes::LaserSword => (0.5, weapon_fire_resource.laser_sword_sprite_render.clone()),
        WeaponFireTypes::Flame => (1.0, weapon_fire_resource.flame_sprite_render.clone()),
        WeaponFireTypes::Grenade => (1.0, weapon_fire_resource.grenade_sprite_render.clone()),
        WeaponFireTypes::Ion => (1.0, weapon_fire_resource.ion_sprite_render.clone()),
    };

    //Player colored weapons
    if weapon_fire_type == WeaponFireTypes::Mine {
        weapon_sprite = get_mine_sprite(player_id, weapon_fire_resource);
    } else if weapon_fire_type == WeaponFireTypes::Trap {
        weapon_sprite = get_trap_sprite(player_id, weapon_fire_resource);
    }

    (icon_scale*3.0, weapon_sprite)
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


pub fn get_weapon_width_height(weapon_fire_type: WeaponFireTypes) -> (f32, f32)
{
    let (width, height) = match weapon_fire_type {
        WeaponFireTypes::LaserDouble => (3.0, 6.0),
        WeaponFireTypes::LaserBeam => (1.0, 12.0),
        WeaponFireTypes::LaserPulse => (2.0, 5.0),
        WeaponFireTypes::ProjectileBurstFire => (1.0, 4.0),
        WeaponFireTypes::ProjectileRapidFire => (1.0, 2.0),
        WeaponFireTypes::ProjectileCannonFire => (3.0, 3.0),
        WeaponFireTypes::Missile => (5.0, 6.0),
        WeaponFireTypes::Rockets => (5.0, 4.0),
        WeaponFireTypes::Mine => (4.0, 4.0),
        WeaponFireTypes::Grenade => (4.0, 4.0),
        WeaponFireTypes::Trap => (2.0, 4.0),
        WeaponFireTypes::LaserSword => (3.0, 15.0),
        WeaponFireTypes::Flame => (6.0, 4.0),
        WeaponFireTypes::Ion => (5.0, 5.0),
    };

    (width, height)
}