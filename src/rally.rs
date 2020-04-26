use amethyst::core::math::Vector3;
use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    prelude::*,
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    ui::{UiText, UiTransform},
};

use crate::audio::initialize_audio;

//use rand::Rng;

use crate::components::{
    weapon_type_from_u8, Hitbox, PlayerWeaponIcon, Vehicle, Weapon, WeaponFire, WeaponTypes,
};

use crate::entities::{initialize_arena_walls, initialize_camera, initialize_ui, intialize_player};

use crate::resources::{initialize_weapon_fire_resource, WeaponFireResource};

pub const ARENA_HEIGHT: f32 = 400.0;
pub const UI_HEIGHT: f32 = 35.0;
pub const ARENA_WIDTH: f32 = 400.0;

pub const BASE_COLLISION_DAMAGE: f32 = 20.0;
pub const COLLISION_PIERCING_DAMAGE_PCT: f32 = 0.0;
pub const COLLISION_SHIELD_DAMAGE_PCT: f32 = 25.0;
pub const COLLISION_ARMOR_DAMAGE_PCT: f32 = 80.0;
pub const COLLISION_HEALTH_DAMAGE_PCT: f32 = 100.0;

pub const MAX_PLAYERS: usize = 4;
pub const BOT_PLAYERS: usize = 3;

pub const KILLS_TO_WIN: i32 = 10;

#[derive(Default)]
pub struct Rally {
    sprite_sheet_handle: Option<Handle<SpriteSheet>>, // Load the spritesheet necessary to render the graphics.
}

impl SimpleState for Rally {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.sprite_sheet_handle.replace(load_sprite_sheet(world));

        initialize_camera(world);

        let weapon_fire_resource: WeaponFireResource =
            initialize_weapon_fire_resource(world, self.sprite_sheet_handle.clone().unwrap());

        initialize_audio(world);

        let player_status_texts = initialize_ui(world);
        world.register::<UiText>(); // <- add this line temporarily
        world.register::<UiTransform>();

        initialize_arena_walls(world, self.sprite_sheet_handle.clone().unwrap());
        world.register::<Hitbox>();

        world.register::<PlayerWeaponIcon>();

        for player_index in 0..MAX_PLAYERS {
            let is_bot = player_index >= MAX_PLAYERS - BOT_PLAYERS;

            intialize_player(
                world,
                self.sprite_sheet_handle.clone().unwrap(),
                player_index,
                weapon_type_from_u8(0),
                weapon_fire_resource.clone(),
                is_bot,
                player_status_texts[player_index],
            );
        }

        //Debug Spawns
        /*
        let weapon1: WeaponTypes = weapon_type_from_u8(0);
        let weapon2: WeaponTypes = weapon_type_from_u8(0);
        let weapon3: WeaponTypes = weapon_type_from_u8(0);
        let weapon4: WeaponTypes = weapon_type_from_u8(0);

        /*
        let mut rng = rand::thread_rng();

        let weapon1: WeaponTypes = weapon_type_from_u8(rng.gen_range(0, 1) as u8);
        let weapon2: WeaponTypes = weapon_type_from_u8(rng.gen_range(0, 9) as u8);
        let weapon3: WeaponTypes = weapon_type_from_u8(rng.gen_range(0, 9) as u8);
        let weapon4: WeaponTypes = weapon_type_from_u8(rng.gen_range(0, 9) as u8);
        */

        intialize_player(
            world,
            self.sprite_sheet_handle.clone().unwrap(),
            0 as usize,
            weapon1,
        );
        intialize_player(
            world,
            self.sprite_sheet_handle.clone().unwrap(),
            1 as usize,
            weapon2,
        );
        intialize_player(
            world,
            self.sprite_sheet_handle.clone().unwrap(),
            2 as usize,
            weapon3,
        );
        intialize_player(
            world,
            self.sprite_sheet_handle.clone().unwrap(),
            3 as usize,
            weapon4,
        );
        */

        //world.register::<Vehicle>(); // <- add this line temporarily
        //world.register::<Weapon>(); // <- add this line temporarily
        //world.register::<WeaponFire>(); // <- add this line temporarily
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        data.world.maintain();

        Trans::None
    }
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `texture_handle` is a cloneable reference to the texture
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/rally_spritesheet.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/rally_spritesheet.ron", // Here we load the associated ron file
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}

pub fn fire_weapon(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    weapon: Weapon,
    fire_position: Vector3<f32>,
    fire_angle: f32,
    player_id: usize,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let fire_entity: Entity = entities.create();

    let mut weapon_fire = WeaponFire::new(
        weapon.weapon_type.clone(),
        player_id,
        weapon.heat_seeking,
        weapon.heat_seeking_agility,
        weapon.attached,
        weapon.deployed,
        weapon.weapon_shot_speed,
        weapon.damage,
        weapon.shield_damage_pct,
        weapon.armor_damage_pct,
        weapon.piercing_damage_pct,
        weapon.health_damage_pct,
    );

    let local_transform = {
        let mut local_transform = Transform::default();
        local_transform.set_translation(fire_position);

        let angle_x_comp: f32 = -fire_angle.sin();
        let angle_y_comp: f32 = fire_angle.cos();

        local_transform.set_rotation_2d(fire_angle);

        weapon_fire.dx = weapon_fire.weapon_shot_speed * angle_x_comp;
        weapon_fire.dy = weapon_fire.weapon_shot_speed * angle_y_comp;

        //adjust the first postion
        let x = local_transform.translation().x;
        let y = local_transform.translation().y;

        //let yaw_width = weapon_fire.height*0.5 * angle_x_comp + weapon_fire.width*0.5 * (1.0-angle_x_comp);
        //let yaw_height = weapon_fire.height*0.5 * angle_y_comp + weapon_fire.width*0.5 * (1.0-angle_y_comp);
        let yaw_width = 0.0;
        let yaw_height = 0.0;

        local_transform.set_translation_x(x - yaw_width);
        local_transform.set_translation_y(y + yaw_height);

        local_transform
    };
    lazy_update.insert(fire_entity, weapon_fire);

    let mut sprite = match weapon.weapon_type.clone() {
        WeaponTypes::LaserDouble => weapon_fire_resource.laser_double_sprite_render.clone(),
        WeaponTypes::LaserBeam => weapon_fire_resource.laser_beam_sprite_render.clone(),
        WeaponTypes::LaserPulse => weapon_fire_resource.laser_burst_sprite_render.clone(),
        WeaponTypes::ProjectileBurstFire => weapon_fire_resource.projectile_burst_render.clone(),
        WeaponTypes::ProjectileRapidFire => weapon_fire_resource.projectile_rapid_render.clone(),
        WeaponTypes::ProjectileCannonFire => {
            weapon_fire_resource.projectile_cannon_sprite_render.clone()
        }
        WeaponTypes::Missile => weapon_fire_resource.missile_sprite_render.clone(),
        WeaponTypes::Rockets => weapon_fire_resource.rockets_sprite_render.clone(),
        WeaponTypes::Mine => weapon_fire_resource.mine_p1_sprite_render.clone(),
        WeaponTypes::LaserSword => weapon_fire_resource.laser_sword_sprite_render.clone(),
    };

    if weapon.weapon_type.clone() == WeaponTypes::Mine {
        sprite = match player_id {
            0 => weapon_fire_resource.mine_p1_sprite_render.clone(),
            1 => weapon_fire_resource.mine_p2_sprite_render.clone(),
            2 => weapon_fire_resource.mine_p3_sprite_render.clone(),
            3 => weapon_fire_resource.mine_p4_sprite_render.clone(),
            _ => weapon_fire_resource.mine_p1_sprite_render.clone(),
        }
    }

    lazy_update.insert(fire_entity, sprite);
    lazy_update.insert(fire_entity, local_transform);
}

pub fn vehicle_damage_model(
    vehicle: &mut Vehicle,
    mut damage: f32,
    piercing_damage_pct: f32,
    shield_damage_pct: f32,
    armor_damage_pct: f32,
    health_damage_pct: f32,
) -> bool {
    let mut piercing_damage: f32 = 0.0;

    if piercing_damage_pct > 0.0 {
        piercing_damage = damage * piercing_damage_pct / 100.0;
        damage -= piercing_damage;
    }

    //println!("H:{:>6.3} A:{:>6.3} S:{:>6.3} P:{:>6.3}, D:{:>6.3}",vehicle.health, vehicle.armor, vehicle.shield, piercing_damage, damage);

    if vehicle.shield > 0.0 {
        vehicle.shield -= damage * shield_damage_pct / 100.0;
        damage = 0.0;

        if vehicle.shield < 0.0 {
            damage -= vehicle.shield; //over damage on shields, needs taken from armor
            vehicle.shield = 0.0;
        } else {
            //take damage to shields, but shields are still alive, reset shield recharge cooldown
            vehicle.shield_cooldown_timer = vehicle.shield_cooldown_reset;
        }
    }

    if vehicle.armor > 0.0 {
        vehicle.armor -= damage * armor_damage_pct / 100.0;
        damage = 0.0;

        if vehicle.armor < 0.0 {
            damage -= vehicle.armor; //over damage on armor, needs taken from health
            vehicle.armor = 0.0;
        }
    }

    let health_damage: f32 = (damage + piercing_damage) * health_damage_pct / 100.0;

    let mut vehicle_destroyed = false;

    if vehicle.health <= health_damage {
        vehicle_destroyed = true;
        vehicle.health = 0.0;
    } else {
        vehicle.health -= health_damage;
    }

    //println!("H:{:>6.3} A:{:>6.3} S:{:>6.3}",vehicle.health, vehicle.armor, vehicle.shield);

    vehicle_destroyed
}

/*
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisBinding {
    VehicleAccel(usize),
    VehicleTurn(usize),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionBinding {
    VehicleShoot(usize),
}

impl Display for AxisBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for ActionBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct MovementBindingTypes;

impl BindingTypes for MovementBindingTypes {
    type Axis = AxisBinding;
    type Action = ActionBinding;
}
*/
