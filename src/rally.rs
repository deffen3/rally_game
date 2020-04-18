use amethyst::{
    assets::{AssetStorage, Loader, Handle},
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    input::{is_key_down, VirtualKeyCode},
};

use std::fmt::{self, Display};

use serde::{Serialize, Deserialize};

use amethyst::input::{BindingTypes};



pub const ARENA_HEIGHT: f32 = 300.0;
pub const ARENA_WIDTH: f32 = 300.0;

pub const VEHICLE_HEIGHT: f32 = 12.0;
pub const VEHICLE_WIDTH: f32 = 6.0;


pub const MAX_PLAYERS: usize = 4;

//testing git

#[derive(Default)]
pub struct Rally {
    sprite_sheet_handle: Option<Handle<SpriteSheet>>, // Load the spritesheet necessary to render the graphics.
}

impl SimpleState for Rally {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.sprite_sheet_handle.replace(load_sprite_sheet(world));

        initialise_camera(world);

        for player_index in 0..MAX_PLAYERS {
            initialize_vehicle(
                world, 
                self.sprite_sheet_handle.clone().unwrap(),
                player_index as usize,
            );
        }

        //world.register::<Vehicle>(); // <- add this line temporarily
        //world.register::<Weapon>(); // <- add this line temporarily
        //world.register::<WeaponFire>(); // <- add this line temporarily
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::None
    }
}


#[derive(Clone)]
pub enum WeaponTypes {
    NoWeapon,
    LaserBeam,
    LaserPulse,
    LaserDouble,
    ProjectileRapidFire,
    ProjectileBurstFire,
    ProjectileSnipeFire,
    Rocket,
    Missile,
    Mine,
}


pub struct Weapon {
    pub x: f32,
    pub y: f32,
    pub aim_angle: f32,
    pub weapon_type: WeaponTypes,
    pub weapon_cooldown_timer: f32,
    pub weapon_cooldown_reset: f32,
    pub weapon_firing: bool,
}

impl Component for Weapon {
    type Storage = DenseVecStorage<Self>;
}

impl Weapon {
    fn new(weapon_type: WeaponTypes) -> Weapon {
        Weapon {
            x: 0.0,
            y: 0.0,
            aim_angle: 0.0,
            weapon_type: weapon_type,
            weapon_cooldown_timer: -1.0,
            weapon_cooldown_reset: 2.0,
            weapon_firing: false,
        }
    }
}


pub struct WeaponFire {
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub spawn_angle: f32,
    pub weapon_type: WeaponTypes,
    pub active: bool,
}

impl Component for WeaponFire {
    type Storage = DenseVecStorage<Self>;
}


impl WeaponFire {
    fn new(weapon_type: WeaponTypes) -> WeaponFire {
        WeaponFire {
            width: 1.0,
            height: 1.0,
            dx: 0.0,
            dy: 0.0,
            spawn_x: 0.0,
            spawn_y: 0.0,
            spawn_angle: 0.0,
            weapon_type: weapon_type,
            active: false,
        }
    }
}



pub struct Vehicle {
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
    pub id: usize,
}

impl Component for Vehicle {
    type Storage = DenseVecStorage<Self>;
}

impl Vehicle {
    fn new(id: usize) -> Vehicle {
        Vehicle {
            width: VEHICLE_WIDTH,
            height: VEHICLE_HEIGHT,
            dx: 0.0,
            dy: 0.0,
            id: id,
        }
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


fn initialize_vehicle(
    world: &mut World, 
    sprite_sheet_handle: Handle<SpriteSheet>,
    ship_index: usize,
) {
let mut local_transform = Transform::default();
local_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT /2.0, 0.0);

// Assign the sprite for the vehicle
let sprite_render = SpriteRender {
    sprite_sheet: sprite_sheet_handle.clone(),
    sprite_number: ship_index,
};

world
    .create_entity()
    .with(sprite_render)
    .with(Vehicle::new(ship_index))
    .with(local_transform)
    .build();
}


fn initialize_weapon(
        world: &mut World, 
        sprite_sheet_handle: Handle<SpriteSheet>,
        weapon_type: WeaponTypes,
    ) {

    let mut weapon_local_transform = Transform::default();
    weapon_local_transform.set_translation_xyz(-ARENA_WIDTH, -ARENA_HEIGHT, 0.0);

    let mut fire_local_transform = Transform::default();
    fire_local_transform.set_translation_xyz(-ARENA_WIDTH, ARENA_HEIGHT, 0.0);

    let fire_sprite_number = match weapon_type.clone() {
        WeaponTypes::NoWeapon => 0 as usize,
        WeaponTypes::LaserBeam => 4 as usize,
        WeaponTypes::LaserPulse => 4 as usize,
        WeaponTypes::LaserDouble => 4 as usize,
        WeaponTypes::ProjectileRapidFire => 5 as usize,
        WeaponTypes::ProjectileBurstFire => 5 as usize,
        WeaponTypes::ProjectileSnipeFire => 5 as usize,
        WeaponTypes::Rocket => 6 as usize,
        WeaponTypes::Missile => 6 as usize,
        WeaponTypes::Mine => 6 as usize,
    };

    let weapon_sprite_number = match weapon_type.clone() {
        WeaponTypes::NoWeapon => 0 as usize,
        WeaponTypes::LaserBeam => 7 as usize,
        WeaponTypes::LaserPulse => 7 as usize,
        WeaponTypes::LaserDouble => 7 as usize,
        WeaponTypes::ProjectileRapidFire => 8 as usize,
        WeaponTypes::ProjectileBurstFire => 8 as usize,
        WeaponTypes::ProjectileSnipeFire => 8 as usize,
        WeaponTypes::Rocket => 9 as usize,
        WeaponTypes::Missile => 9 as usize,
        WeaponTypes::Mine => 9 as usize,
    };

    if weapon_sprite_number > 0 {
        let weapon_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: weapon_sprite_number,
        };

        world
            .create_entity()
            .with(weapon_sprite_render)
            .with(Weapon::new(weapon_type.clone()))
            .with(weapon_local_transform)
            .build();


        // Assign the sprite for the weapon fire
        let fire_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle,
            sprite_number: fire_sprite_number,
        };

        world
            .create_entity()
            .with(fire_sprite_render)
            .with(WeaponFire::new(weapon_type.clone()))
            .with(fire_local_transform)
            .build();
    }
}


fn initialise_camera(world: &mut World) {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left. 
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();
}




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