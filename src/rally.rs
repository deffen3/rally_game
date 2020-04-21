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

use crate::audio::initialise_audio;

use rand::Rng;



pub const ARENA_HEIGHT: f32 = 400.0;
pub const ARENA_WIDTH: f32 = 400.0;

pub const VEHICLE_HEIGHT: f32 = 12.0;
pub const VEHICLE_WIDTH: f32 = 6.0;

pub const COLLISION_DAMAGE: f32 = 20.0;

pub const MAX_PLAYERS: usize = 4;


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

        initialise_audio(world);

        // for player_index in 0..MAX_PLAYERS {
        //     intialize_player(
        //         world, 
        //         self.sprite_sheet_handle.clone().unwrap(),
        //         player_index as usize,
        //         WeaponTypes::ProjectileCannonFire,
        //     );
        // }

        let mut rng = rand::thread_rng();

        /*
        let weapon1: WeaponTypes = weapon_type_from_u8(6);
        let weapon2: WeaponTypes = weapon_type_from_u8(7);
        let weapon3: WeaponTypes = weapon_type_from_u8(8);
        let weapon4: WeaponTypes = weapon_type_from_u8(100);
        */
        
        let weapon1: WeaponTypes = weapon_type_from_u8(rng.gen_range(0, 9) as u8);
        let weapon2: WeaponTypes = weapon_type_from_u8(rng.gen_range(0, 9) as u8);
        let weapon3: WeaponTypes = weapon_type_from_u8(rng.gen_range(0, 9) as u8);
        let weapon4: WeaponTypes = weapon_type_from_u8(rng.gen_range(0, 9) as u8);
        

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
        // intialize_player(
        //     world, 
        //     self.sprite_sheet_handle.clone().unwrap(),
        //     2 as usize,
        //     weapon3,
        // );
        // intialize_player(
        //     world, 
        //     self.sprite_sheet_handle.clone().unwrap(),
        //     3 as usize,
        //     weapon4,
        // );


        //world.register::<Vehicle>(); // <- add this line temporarily
        //world.register::<Weapon>(); // <- add this line temporarily
        //world.register::<WeaponFire>(); // <- add this line temporarily
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        data.world.maintain();

        Trans::None
    }
}


fn weapon_type_from_u8(n: u8) -> WeaponTypes {
    match n {
        0 => WeaponTypes::LaserBeam,
        1 => WeaponTypes::LaserPulse,
        2 => WeaponTypes::LaserDouble,
        3 => WeaponTypes::ProjectileRapidFire,
        4 => WeaponTypes::ProjectileBurstFire,
        5 => WeaponTypes::ProjectileCannonFire,
        6 => WeaponTypes::Mine,
        7 => WeaponTypes::Missile,
        8 => WeaponTypes::Rockets,
        _ => WeaponTypes::LaserBeam,
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


#[derive(Clone)]
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
    pub damage: f32,
    pub weapon_shot_speed: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
}

impl Component for Weapon {
    type Storage = DenseVecStorage<Self>;
}

impl Weapon {
    fn new(weapon_type: WeaponTypes, 
        weapon_cooldown: f32, 
        burst_shot_limit: u32, 
        burst_cooldown: f32,
        weapon_shot_speed: f32,
        damage: f32,
        shield_damage_pct: f32,
        armor_damage_pct: f32,
        piercing_damage_pct: f32,
        health_damage_pct: f32,
    ) -> Weapon {

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
            damage: damage,
            weapon_shot_speed: weapon_shot_speed,
            shield_damage_pct: shield_damage_pct,
            armor_damage_pct: armor_damage_pct,
            piercing_damage_pct: piercing_damage_pct,
            health_damage_pct: health_damage_pct,
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
    pub weapon_shot_speed: f32,
    pub owner_player_id: usize,
    pub damage: f32,
    pub shield_damage_pct: f32,
    pub armor_damage_pct: f32,
    pub piercing_damage_pct: f32,
    pub health_damage_pct: f32,
    pub weapon_type: WeaponTypes,
}

impl Component for WeaponFire {
    type Storage = DenseVecStorage<Self>;
}


impl WeaponFire {
    fn new(weapon_type: WeaponTypes, 
        owner_player_id: usize,
        weapon_shot_speed: f32,
        damage: f32,
        shield_damage_pct: f32,
        armor_damage_pct: f32,
        piercing_damage_pct: f32,
        health_damage_pct: f32,
    ) -> WeaponFire {

        WeaponFire {
            width: 1.0,
            height: 1.0,
            dx: 0.0,
            dy: 0.0,
            spawn_x: 0.0,
            spawn_y: 0.0,
            spawn_angle: 0.0,
            owner_player_id,
            damage: damage,
            weapon_shot_speed: weapon_shot_speed,
            shield_damage_pct: shield_damage_pct,
            armor_damage_pct: armor_damage_pct,
            piercing_damage_pct: piercing_damage_pct,
            health_damage_pct: health_damage_pct,
            weapon_type,
        }
    }
}





pub struct Vehicle {
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
    pub dr: f32,
    pub collision_cooldown_timer: f32,
    pub health: f32,
    pub shield: f32,
    pub armor: f32,
    pub weight: f32,
    pub engine_power: f32
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
            dr: 0.0,
            collision_cooldown_timer: -1.0,
            health: 100.0,
            shield: 100.0,
            armor: 100.0,
            weight: 100.0,
            engine_power: 100.0,
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

    vehicle_transform.set_rotation_2d(0.0 as f32);
    vehicle_transform.set_translation_xyz(ARENA_WIDTH / 5.0 * ((player_index + 1) as f32), ARENA_HEIGHT /2.0, 0.0);

    let vehicle_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: player_index,
    };

    let (weapon_type,
        weapon_cooldown, 
        burst_shot_limit,
        burst_cooldown,
        weapon_shot_speed,
        damage,
        shield_damage_pct,
        armor_damage_pct,
        piercing_damage_pct,
        health_damage_pct,) = build_standard_weapon(weapon_type);

    world
        .create_entity()
        .with(vehicle_transform)
        .with(vehicle_sprite_render)
        .with(Vehicle::new())
        .with(Weapon::new(weapon_type,
            weapon_cooldown, 
            burst_shot_limit,
            burst_cooldown,
            weapon_shot_speed,
            damage,
            shield_damage_pct,
            armor_damage_pct,
            piercing_damage_pct,
            health_damage_pct))
        .with(Player::new(player_index))
        .build();


    //I can build all of this as one entity, but then I get only one sprite.
    //if I separate it into three entities, then now my systems are broken as their
    //  is no relationship between these entities. Do I need to apply parent child relationships?
    //  Isn't this going against the purpose/elegance of ECS?
}




fn build_standard_weapon(weapon_type: WeaponTypes) -> (
    WeaponTypes, f32, u32, f32, f32, f32, f32, f32, f32, f32
) {
    let (weapon_shot_speed, damage, weapon_cooldown, 
            piercing_damage_pct, 
            shield_damage_pct, armor_damage_pct, 
            health_damage_pct,
        ) = match weapon_type.clone()
    {                                      //speed      dmg     cool    pierce% shield%   armor%    health%
        WeaponTypes::LaserDouble =>         (400.0,     25.0,   0.4,    0.0,    125.0,    80.0,     100.0),
        WeaponTypes::LaserBeam =>           (2800.0,    0.3,    0.0,    0.0,    125.0,    80.0,     100.0),
        WeaponTypes::LaserPulse =>          (400.0,     12.0,   0.75,   0.0,    125.0,    80.0,     100.0),
        WeaponTypes::ProjectileBurstFire => (250.0,     12.0,   0.15,   0.0,    100.0,    100.0,    100.0),
        WeaponTypes::ProjectileRapidFire => (250.0,     3.0,    0.9,    0.0,    100.0,    100.0,    100.0),
        WeaponTypes::ProjectileCannonFire =>(700.0,     50.0,   0.0,    0.0,    100.0,    100.0,    100.0),
        WeaponTypes::Missile =>             (100.0,     50.0,   2.5,    50.0,   100.0,    100.0,    100.0),
        WeaponTypes::Rockets =>             (250.0,     50.0,   0.5,    50.0,   100.0,    100.0,    100.0),
        WeaponTypes::Mine =>                (0.0,       50.0,   2.5,    50.0,   100.0,    100.0,    100.0),
    };

    let mut burst_cooldown;
    let mut burst_shot_limit; 
    if weapon_type.clone() == WeaponTypes::LaserPulse {
        burst_cooldown = 0.1 as f32;
        burst_shot_limit = 2 as u32;
    }
    else if weapon_type.clone() == WeaponTypes::ProjectileBurstFire{
        burst_cooldown = 0.1 as f32;
        burst_shot_limit = 2 as u32;
    }
    else {
        burst_cooldown = weapon_cooldown.clone();
        burst_shot_limit = 1 as u32;
    };

    (weapon_type,
        weapon_cooldown, 
        burst_shot_limit,
        burst_cooldown,
        weapon_shot_speed,
        damage,
        shield_damage_pct,
        armor_damage_pct,
        piercing_damage_pct,
        health_damage_pct,)
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

    let sprite = match weapon.weapon_type {
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






pub fn vehicle_damage_model(vehicle: &mut Vehicle, 
        mut damage:f32, piercing_damage_pct:f32, 
        shield_damage_pct:f32, armor_damage_pct:f32, health_damage_pct:f32
    ) -> bool {

    let mut piercing_damage:f32 = 0.0;

    if piercing_damage_pct > 0.0 {
        piercing_damage = damage * piercing_damage_pct/100.0;
        damage -= piercing_damage;
    }

    println!("H:{} A:{} S:{} P:{}, D:{}",vehicle.health, vehicle.armor, vehicle.shield, piercing_damage, damage);

    if vehicle.shield > 0.0 {
        vehicle.shield -= (damage * shield_damage_pct/100.0);
        damage = 0.0;

        if vehicle.shield < 0.0 {
            damage -= vehicle.shield; //over damage on shields, needs taken from armor
            vehicle.shield = 0.0;
        }
    }

    println!("H:{} A:{} S:{} D:{}",vehicle.health, vehicle.armor, vehicle.shield, damage);

    if vehicle.armor > 0.0 {
        vehicle.armor -= (damage * armor_damage_pct/100.0);
        damage = 0.0;

        if vehicle.armor < 0.0 {
            damage -= vehicle.armor; //over damage on armor, needs taken from health
            vehicle.armor = 0.0;
        }
    }

    println!("H:{} A:{} S:{} D:{}",vehicle.health, vehicle.armor, vehicle.shield, damage);

    let mut health_damage:f32 = (damage + piercing_damage) * health_damage_pct/100.0;

    let mut vehicle_destroyed = false;

    if vehicle.health <= health_damage {
        vehicle_destroyed = true;
        vehicle.health = 0.0;
    }
    else {
        vehicle.health -= health_damage;
        health_damage = 0.0;
    }

    println!("H:{} A:{} S:{} D:{}",vehicle.health, vehicle.armor, vehicle.shield, health_damage);

    vehicle_destroyed
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