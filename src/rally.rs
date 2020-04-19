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

        // for player_index in 0..MAX_PLAYERS {
        //     intialize_player(
        //         world, 
        //         self.sprite_sheet_handle.clone().unwrap(),
        //         player_index as usize,
        //         WeaponTypes::ProjectileCannonFire,
        //     );
        // }

        intialize_player(
            world, 
            self.sprite_sheet_handle.clone().unwrap(),
            0 as usize,
            WeaponTypes::ProjectileCannonFire,
        );
        intialize_player(
            world, 
            self.sprite_sheet_handle.clone().unwrap(),
            1 as usize,
            WeaponTypes::Missile,
        );
        intialize_player(
            world, 
            self.sprite_sheet_handle.clone().unwrap(),
            2 as usize,
            WeaponTypes::Rockets,
        );
        intialize_player(
            world, 
            self.sprite_sheet_handle.clone().unwrap(),
            3 as usize,
            WeaponTypes::Mine,
        );


        //world.register::<Vehicle>(); // <- add this line temporarily
        //world.register::<Weapon>(); // <- add this line temporarily
        //world.register::<WeaponFire>(); // <- add this line temporarily
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        data.world.maintain();

        Trans::None
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


pub struct Weapon {
    pub x: f32,
    pub y: f32,
    pub aim_angle: f32,
    pub weapon_type: WeaponTypes,
    pub weapon_cooldown_timer: f32,
    pub weapon_cooldown_reset: f32,
    pub burst_shots: u32,
    pub burst_shot_limit: u32,
    pub burst_cooldown_reset: f32,
}

impl Component for Weapon {
    type Storage = DenseVecStorage<Self>;
}

impl Weapon {
    fn new(weapon_type: WeaponTypes) -> Weapon {
        let weapon_cooldown = match weapon_type.clone() {
            WeaponTypes::LaserDouble => 0.5,
            WeaponTypes::LaserBeam => 0.0,
            WeaponTypes::LaserPulse => 1.0,
            WeaponTypes::ProjectileBurstFire => 0.5,
            WeaponTypes::ProjectileRapidFire => 0.2,
            WeaponTypes::ProjectileCannonFire => 1.0,
            WeaponTypes::Missile => 3.0,
            WeaponTypes::Rockets => 3.0,
            WeaponTypes::Mine => 3.0,
        };

        let mut burst_cooldown;
        let mut burst_shot_limit; 
        if weapon_type.clone() == WeaponTypes::LaserPulse {
            burst_cooldown = 0.2 as f32;
            burst_shot_limit = 2 as u32;
        }
        else if weapon_type.clone() == WeaponTypes::ProjectileBurstFire{
            burst_cooldown = 0.2 as f32;
            burst_shot_limit = 2 as u32;
        }
        else {
            burst_cooldown = weapon_cooldown.clone();
            burst_shot_limit = 1 as u32;
        };

        Weapon {
            x: 0.0,
            y: 0.0,
            aim_angle: 0.0,
            weapon_type,
            weapon_cooldown_timer: -1.0,
            weapon_cooldown_reset: weapon_cooldown,
            burst_shots: 0,
            burst_shot_limit: burst_shot_limit,
            burst_cooldown_reset: burst_cooldown,
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
    pub speed: f32,
    pub weapon_type: WeaponTypes,
}

impl Component for WeaponFire {
    type Storage = DenseVecStorage<Self>;
}


impl WeaponFire {
    fn new(weapon_type: WeaponTypes) -> WeaponFire {
        let weapon_shot_speed = match weapon_type.clone() {
            WeaponTypes::LaserDouble => 400.0,
            WeaponTypes::LaserBeam => 2800.0,
            WeaponTypes::LaserPulse => 400.0,
            WeaponTypes::ProjectileBurstFire => 250.0,
            WeaponTypes::ProjectileRapidFire => 250.0,
            WeaponTypes::ProjectileCannonFire => 700.0,
            WeaponTypes::Missile => 100.0,
            WeaponTypes::Rockets => 250.0,
            WeaponTypes::Mine => 0.0,
        };

        WeaponFire {
            width: 1.0,
            height: 3.0,
            dx: 0.0,
            dy: 0.0,
            spawn_x: 0.0,
            spawn_y: 0.0,
            spawn_angle: 0.0,
            speed: weapon_shot_speed,
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
    /// The render that locates the sprite in a sprite sheet resource
    pub laser_double_sprite_render: SpriteRender,
    pub laser_beam_sprite_render: SpriteRender,
    pub laser_burst_sprite_render: SpriteRender,
    pub projectile_cannon_sprite_render: SpriteRender,
    pub projectile_burst_render: SpriteRender,
    pub projectile_rapid_render: SpriteRender,
    pub mine_sprite_render: SpriteRender,
    pub missile_sprite_render: SpriteRender,
    pub rockets_sprite_render: SpriteRender,
}



pub fn initialise_weapon_fire_resource(
    world: &mut World,
    sprite_sheet_handle: Handle<SpriteSheet>,
) -> WeaponFireResource {
    let weapon_fire_resource = WeaponFireResource {
        laser_double_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 4,
        },
        laser_beam_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 5,
        },
        laser_burst_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 6,
        },
        projectile_cannon_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 7,
        },
        projectile_burst_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 8,
        },
        projectile_rapid_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 9,
        },
        mine_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 10,
        },
        missile_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 11,
        },
        rockets_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 12,
        },
    };
    world.insert(weapon_fire_resource.clone());
    weapon_fire_resource
}


pub fn fire_weapon(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    weapon_type: WeaponTypes,
    fire_position: Vector3<f32>,
    fire_angle: f32,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let fire_entity: Entity = entities.create();
    let mut weapon_fire = WeaponFire::new(weapon_type.clone());

    let local_transform = {
        let mut local_transform = Transform::default();
        local_transform.set_translation(fire_position);

        let angle_x_comp: f32 = -fire_angle.sin(); //left is -, right is +
        let angle_y_comp: f32 = fire_angle.cos(); //up is +, down is -

        local_transform.set_rotation_2d(fire_angle);

        weapon_fire.dx = weapon_fire.speed * angle_x_comp;
        weapon_fire.dy = weapon_fire.speed * angle_y_comp;
        
        // the fire position actually represents the middle of our laser. Adjust accordingly.
        let x = local_transform.translation().x;
        let y = local_transform.translation().y;

        // local_transform.set_translation_x(x - (weapon_fire.width * angle_x_comp / 2.0));
        // local_transform.set_translation_y(y + (weapon_fire.height * angle_y_comp));
        local_transform
    };
    lazy_update.insert(fire_entity, weapon_fire);

    let sprite = match weapon_type {
        WeaponTypes::LaserDouble => weapon_fire_resource.laser_double_sprite_render.clone(),
        WeaponTypes::LaserBeam => weapon_fire_resource.laser_beam_sprite_render.clone(),
        WeaponTypes::LaserPulse => weapon_fire_resource.laser_burst_sprite_render.clone(),
        WeaponTypes::ProjectileBurstFire => weapon_fire_resource.projectile_burst_render.clone(),
        WeaponTypes::ProjectileRapidFire => weapon_fire_resource.projectile_rapid_render.clone(),
        WeaponTypes::ProjectileCannonFire => weapon_fire_resource.projectile_cannon_sprite_render.clone(),
        WeaponTypes::Missile => weapon_fire_resource.missile_sprite_render.clone(),
        WeaponTypes::Rockets => weapon_fire_resource.rockets_sprite_render.clone(),
        WeaponTypes::Mine => weapon_fire_resource.mine_sprite_render.clone(),
    };
    lazy_update.insert(fire_entity, sprite);
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