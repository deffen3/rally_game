use amethyst::{
    assets::{AssetStorage, Loader, Handle},
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entity, Entities, ReadExpect, LazyUpdate},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
};
use amethyst::core::math::Vector3;

use std::fmt::{self, Display};

use serde::{Serialize, Deserialize};

use amethyst::input::{BindingTypes};



pub const ARENA_HEIGHT: f32 = 300.0;
pub const ARENA_WIDTH: f32 = 300.0;

pub const VEHICLE_HEIGHT: f32 = 12.0;
pub const VEHICLE_WIDTH: f32 = 6.0;

pub const LASER_SPEED: f32 = 3.7;

pub const WEAPON_COOLDOWN: f32 = 0.5;


pub const MAX_PLAYERS: usize = 2;


#[derive(Default)]
pub struct Rally {
    sprite_sheet_handle: Option<Handle<SpriteSheet>>, // Load the spritesheet necessary to render the graphics.
}

impl SimpleState for Rally {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.sprite_sheet_handle.replace(load_sprite_sheet(world));

        initialise_camera(world);
        initialise_weapon_fire_resource(world, self.sprite_sheet_handle.clone().unwrap());

        for player_index in 0..MAX_PLAYERS {
            intialize_player(
                world, 
                self.sprite_sheet_handle.clone().unwrap(),
                player_index as usize,
                WeaponTypes::LaserDouble,
            );
        }

        //world.register::<Vehicle>(); // <- add this line temporarily
        //world.register::<Weapon>(); // <- add this line temporarily
        //world.register::<WeaponFire>(); // <- add this line temporarily
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        data.world.maintain();

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
            weapon_type,
            weapon_cooldown_timer: -1.0,
            weapon_cooldown_reset: WEAPON_COOLDOWN,
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
    pub weapon_type: WeaponTypes,
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
            weapon_type,
        }
    }
}



pub struct Vehicle {
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
}

impl Component for Vehicle {
    type Storage = DenseVecStorage<Self>;
}

impl Vehicle {
    fn new() -> Vehicle {
        Vehicle {
            width: VEHICLE_WIDTH,
            height: VEHICLE_HEIGHT,
            dx: 0.0,
            dy: 0.0,
        }
    }
}



pub struct Player {
    pub id: usize,
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

impl Player {
    fn new(id: usize) -> Player {
        Player {
            id,
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




fn intialize_player(
        world: &mut World, 
        sprite_sheet_handle: Handle<SpriteSheet>,
        player_index: usize,
        weapon_type: WeaponTypes,
    ) {
        

    let mut vehicle_transform = Transform::default();
    vehicle_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT /2.0, 0.0);

    let vehicle_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: player_index,
    };

    /*
    let mut weapon_transform = Transform::default();
    weapon_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT /2.0, 0.0);

    let weapon_sprite_number = match weapon_type.clone() {
        WeaponTypes::NoWeapon => 7 as usize,
        WeaponTypes::LaserBeam => 8 as usize,
        WeaponTypes::LaserPulse => 8 as usize,
        WeaponTypes::LaserDouble => 8 as usize,
        WeaponTypes::ProjectileRapidFire => 9 as usize,
        WeaponTypes::ProjectileBurstFire => 9 as usize,
        WeaponTypes::ProjectileSnipeFire => 9 as usize,
        WeaponTypes::Rocket => 10 as usize,
        WeaponTypes::Missile => 10 as usize,
        WeaponTypes::Mine => 10 as usize,
    };

    let weapon_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: weapon_sprite_number,
    };


    let weapon = world
        .create_entity()
        .with(weapon_sprite_render)
        .with(Weapon::new(weapon_type))
        .with(weapon_transform)
        .build();
    */

    world
        .create_entity()
        .with(vehicle_transform)
        .with(vehicle_sprite_render)
        .with(Vehicle::new())
        .with(Weapon::new(weapon_type))
        .with(Player::new(player_index))
        .build();


    //I can build all of this as one entity, but then I get only one sprite.
    //if I separate it into three entities, then now my systems are broken as their
    //  is no relationship between these entities. Do I need to apply parent child relationships?
    //  Isn't this going against the purpose/elegance of ECS?
}




#[derive(Clone)]
pub struct WeaponFireResource {
    /// The component used to create a laser entity
    pub component: WeaponFire,
    /// The render that locates the sprite in a sprite sheet resource
    pub sprite_render: SpriteRender,
}



pub fn initialise_weapon_fire_resource(
    world: &mut World,
    sprite_sheet_handle: Handle<SpriteSheet>,
) -> WeaponFireResource {
    let weapon_fire_resource = WeaponFireResource {
        component: WeaponFire::new(WeaponTypes::LaserDouble),
        sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 4,
        },
    };
    world.insert(weapon_fire_resource.clone());
    weapon_fire_resource
}


pub fn fire_weapon(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    fire_position: Vector3<f32>,
    fire_angle: f32,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let fire_entity: Entity = entities.create();
    let mut weapon_fire = weapon_fire_resource.component.clone();

    let local_transform = {
        let mut local_transform = Transform::default();
        local_transform.set_translation(fire_position);

        let angle_x_comp: f32 = -fire_angle.sin(); //left is -, right is +
        let angle_y_comp: f32 = fire_angle.cos(); //up is +, down is -

        local_transform.set_rotation_2d(fire_angle);

        weapon_fire.dx = LASER_SPEED * angle_x_comp;
        weapon_fire.dy = LASER_SPEED * angle_y_comp;

        // the fire position actually represents the middle of our laser. Adjust accordingly.
        let p = local_transform.translation()[0];
        local_transform.set_translation_x(p - (weapon_fire.width / 2.0));
        local_transform
    };
    lazy_update.insert(fire_entity, weapon_fire);
    lazy_update.insert(fire_entity, weapon_fire_resource.sprite_render.clone());
    lazy_update.insert(fire_entity, local_transform);
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