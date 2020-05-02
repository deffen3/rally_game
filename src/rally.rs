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
    Hitbox, PlayerWeaponIcon, Vehicle, 
    Weapon, WeaponFire, WeaponNames, get_weapon_icon
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
pub const BOT_PLAYERS: usize = MAX_PLAYERS - 1;

pub const KILLS_TO_WIN: i32 = 15;

#[derive(Default)]
pub struct Rally {
    sprite_sheet_handle: Option<Handle<SpriteSheet>>, // Load the spritesheet necessary to render the graphics.
    texture_sheet_handle: Option<Handle<SpriteSheet>>,
}

impl SimpleState for Rally {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.sprite_sheet_handle.replace(load_sprite_sheet(
            world, "texture/rally_spritesheet.png".to_string(), "texture/rally_spritesheet.ron".to_string()
        ));
        self.texture_sheet_handle.replace(load_sprite_sheet(
            world, "texture/rally_texture_sheet.png".to_string(), "texture/rally_texture_sheet.ron".to_string()
        ));

        initialize_camera(world);

        let weapon_fire_resource: WeaponFireResource =
            initialize_weapon_fire_resource(world, self.sprite_sheet_handle.clone().unwrap());

        initialize_audio(world);

        let player_status_texts = initialize_ui(world);
        world.register::<UiText>(); // <- add this line temporarily
        world.register::<UiTransform>();

        initialize_arena_walls(
            world,
            self.sprite_sheet_handle.clone().unwrap(),
            self.texture_sheet_handle.clone().unwrap(),
        );

        world.register::<Hitbox>();

        world.register::<PlayerWeaponIcon>();

        for player_index in 0..MAX_PLAYERS {
            let is_bot = player_index >= MAX_PLAYERS - BOT_PLAYERS;

            intialize_player(
                world,
                self.sprite_sheet_handle.clone().unwrap(),
                player_index,
                WeaponNames::LaserDoubleGimballed,
                weapon_fire_resource.clone(),
                is_bot,
                player_status_texts[player_index],
            );
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        data.world.maintain();

        Trans::None
    }
}

fn load_sprite_sheet(world: &mut World, storage: String, store: String) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `texture_handle` is a cloneable reference to the texture
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            storage,
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        store, // Here we load the associated ron file
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
        weapon.name.clone(),
        weapon.stats.weapon_type,
        player_id,
        weapon.stats.heat_seeking,
        weapon.stats.heat_seeking_agility,
        weapon.stats.attached,
        weapon.stats.deployed,
        weapon.stats.mounted_angle,
        weapon.stats.shot_speed,
        weapon.stats.shot_life_limit,
        weapon.stats.damage,
        weapon.stats.shield_damage_pct,
        weapon.stats.armor_damage_pct,
        weapon.stats.piercing_damage_pct,
        weapon.stats.health_damage_pct,
    );

    let local_transform = {
        let mut local_transform = Transform::default();
        local_transform.set_translation(fire_position);

        let angle_x_comp: f32 = -fire_angle.sin();
        let angle_y_comp: f32 = fire_angle.cos();

        local_transform.set_rotation_2d(fire_angle);

        weapon_fire.dx = weapon_fire.shot_speed * angle_x_comp;
        weapon_fire.dy = weapon_fire.shot_speed * angle_y_comp;

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


    let (_icon_scale, weapon_sprite) = get_weapon_icon(player_id, weapon.stats, weapon_fire_resource);

    lazy_update.insert(fire_entity, weapon_sprite);
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

    if vehicle.shield.value > 0.0 {
        vehicle.shield.value -= damage * shield_damage_pct / 100.0;
        damage = 0.0;

        if vehicle.shield.value < 0.0 {
            damage -= vehicle.shield.value; //over damage on shields, needs taken from armor
            vehicle.shield.value = 0.0;
        } else {
            //take damage to shields, but shields are still alive, reset shield recharge cooldown
            vehicle.shield.cooldown_timer = vehicle.shield.cooldown_reset;
        }
    }

    if vehicle.armor.value > 0.0 {
        vehicle.armor.value -= damage * armor_damage_pct / 100.0;
        damage = 0.0;

        if vehicle.armor.value < 0.0 {
            damage -= vehicle.armor.value; //over damage on armor, needs taken from health
            vehicle.armor.value = 0.0;
        }
    }

    let health_damage: f32 = (damage + piercing_damage) * health_damage_pct / 100.0;

    let mut vehicle_destroyed = false;

    if vehicle.health.value > 0.0 { //only destroy once
        if vehicle.health.value <= health_damage {
            vehicle_destroyed = true;
            vehicle.health.value = 0.0;
        } else {
            vehicle.health.value -= health_damage;
        }
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
