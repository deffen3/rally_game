use amethyst::{
    core::math::Vector3,
    core::transform::Transform,
    core::Time,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect, 
        DispatcherBuilder, Dispatcher},
    prelude::*,
    ui::{UiText, UiFinder, UiTransform, UiCreator},
    input::{is_close_requested, is_key_down},
    utils::{fps_counter::FpsCounter, removal::{Removal, exec_removal}},
    winit::VirtualKeyCode,
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    assets::{AssetStorage, Handle, Loader},
};

use crate::pause::PauseMenuState;

use crate::resources::{initialize_weapon_fire_resource, WeaponFireResource, GameModeSetup};

use crate::entities::{
    initialize_camera, initialize_camera_to_player,
    initialize_arena_walls, initialize_ui, intialize_player
};

use crate::components::{
    Armor, Health, Hitbox, Player, Repair, Shield, Vehicle, 
    Weapon, WeaponFire,
    PlayerWeaponIcon, get_weapon_icon,
};

use crate::systems::{
    VehicleTrackingSystem,
    VehicleMoveSystem,
    VehicleWeaponsSystem,
    MoveWeaponFireSystem,
    CollisionVehToVehSystem,
    CollisionVehicleWeaponFireSystem,
    VehicleShieldArmorHealthSystem,
    VehicleStatusSystem,
};


pub const PLAYER_CAMERA: bool = false;



pub const ARENA_HEIGHT: f32 = 400.0;
pub const UI_HEIGHT: f32 = 35.0;
pub const ARENA_WIDTH: f32 = 400.0;

pub const BASE_COLLISION_DAMAGE: f32 = 20.0;
pub const COLLISION_PIERCING_DAMAGE_PCT: f32 = 0.0;
pub const COLLISION_SHIELD_DAMAGE_PCT: f32 = 25.0;
pub const COLLISION_ARMOR_DAMAGE_PCT: f32 = 80.0;
pub const COLLISION_HEALTH_DAMAGE_PCT: f32 = 100.0;



#[derive(Default)]
pub struct GameplayState<'a, 'b> {
    // // If the Game is paused or not
    pub paused: bool,
    // The UI root entity. Deleting this should remove the complete UI
    ui_root: Option<Entity>,
    // A reference to the FPS display, which we want to interact with
    fps_display: Option<Entity>,

    /// The `State` specific `Dispatcher`, containing `System`s only relevant for this `State`.
    dispatcher: Option<Dispatcher<'a, 'b>>,

    sprite_sheet_handle: Option<Handle<SpriteSheet>>, // Load the spritesheet necessary to render the graphics.
    texture_sheet_handle: Option<Handle<SpriteSheet>>,
}


impl<'a, 'b> SimpleState for GameplayState<'a, 'b> {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = &mut data.world;

        world.register::<UiText>();
        world.register::<UiTransform>();

        world.register::<Armor>();
        world.register::<Health>();
        world.register::<Hitbox>();
        world.register::<Player>();
        world.register::<Repair>();
        world.register::<Shield>();
        world.register::<Vehicle>();
        world.register::<Weapon>();
        world.register::<WeaponFire>();
        
        world.register::<PlayerWeaponIcon>();

        world.register::<Removal<u32>>();


        self.sprite_sheet_handle.replace(load_sprite_sheet(
            world, "texture/rally_spritesheet.png".to_string(), "texture/rally_spritesheet.ron".to_string()
        ));
        self.texture_sheet_handle.replace(load_sprite_sheet(
            world, "texture/rally_texture_sheet.png".to_string(), "texture/rally_texture_sheet.ron".to_string()
        ));


        let weapon_fire_resource: WeaponFireResource =
            initialize_weapon_fire_resource(world, self.sprite_sheet_handle.clone().unwrap());

    
        let player_status_texts = initialize_ui(world);

        
        let max_players;
        let bot_players;
        {
            let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();

            if let Some(game_mode_setup) = fetched_game_mode_setup {
                max_players = game_mode_setup.max_players;
                bot_players = game_mode_setup.bot_players;
            }
            else {
                max_players = 4;
                bot_players = 3;
            }
        }

        initialize_arena_walls(
            world,
            self.sprite_sheet_handle.clone().unwrap(),
            self.texture_sheet_handle.clone().unwrap(),
        );

        for player_index in 0..max_players {
            let is_bot = player_index >= max_players - bot_players;

            let max_health = 100.0;
            let max_armor = 100.0;
            let max_shield = 100.0;

            let engine_force = 100.0;
            let engine_efficiency = 1.0;
            let engine_weight = engine_force / engine_efficiency * 20./100.;
            
            //stock vehicle weight at 100/100/100 with normal engine efficiency is 100

            //health makes up the main hull of the vehicle, and contributes 30 base + 10per health weight
            //shields make up 15 weight
            //armor another 25 weight
            //engine another 20 weight

            //typical weapon weight adds about 10.0

            let max_velocity = 1.0;


            let player = intialize_player(
                world,
                self.sprite_sheet_handle.clone().unwrap(),
                player_index,
                weapon_fire_resource.clone(),
                is_bot,
                player_status_texts[player_index],
                max_health,
                max_armor,
                max_shield,
                engine_force,
                engine_weight,
                max_velocity,
            );

            if PLAYER_CAMERA && !is_bot {
                initialize_camera_to_player(world, player);
            }
        }

        if !PLAYER_CAMERA {
            initialize_camera(world);
        }


        // Create the `DispatcherBuilder` and register some `System`s that should only run for this `State`.
        let mut dispatcher_builder = DispatcherBuilder::new();
        dispatcher_builder.add(VehicleTrackingSystem, 
            "vehicle_tracking_system", &[]);
        dispatcher_builder.add(VehicleMoveSystem::default(), 
            "vehicle_move_system", &[]);

        dispatcher_builder.add(VehicleWeaponsSystem, 
            "vehicle_weapons_system", &[]);
        dispatcher_builder.add(MoveWeaponFireSystem, 
            "move_weapon_fire_system", &["vehicle_weapons_system"]);

        dispatcher_builder.add(CollisionVehToVehSystem,
            "collision_vehicle_vehicle_system", &["vehicle_move_system"]);
        dispatcher_builder.add(CollisionVehicleWeaponFireSystem::default(),
            "collision_vehicle_weapon_fire_system", &["vehicle_move_system", "move_weapon_fire_system"]);

        dispatcher_builder.add(VehicleShieldArmorHealthSystem,
            "vehicle_shield_armor_health_system", &[]);
        dispatcher_builder.add(VehicleStatusSystem::default(),
            "vehicle_status_system", &[]);


        // Build and setup the `Dispatcher`.
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(world);
 
        self.dispatcher = Some(dispatcher);

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/game_fps.ron", ())));
    }

    fn on_pause(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        self.paused = true;
    }

    fn on_resume(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        self.paused = false;
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if let Some(root_entity) = self.ui_root {
            data.world
                .delete_entity(root_entity)
                .expect("Failed to remove Game Screen");
        }

        exec_removal(&data.world.entities(), &data.world.read_storage(), 0 as u32);

        self.ui_root = None;
        self.fps_display = None;
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Push] Pausing Game!");
                    Trans::Push(Box::new(PauseMenuState::default()))
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(_ui_event) => {
                // log::info!(
                //     "[HANDLE_EVENT] You just interacted with a ui element: {:?}",
                //     ui_event
                // );
                Trans::None
            }
            StateEvent::Input(_input) => {
                //log::info!("Input Event detected: {:?}.", input);
                Trans::None
            }
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        let world = &mut data.world;

        // this cannot happen in 'on_start', as the entity might not be fully
        // initialized/registered/created yet.
        if self.fps_display.is_none() {
            world.exec(|finder: UiFinder<'_>| {
                if let Some(entity) = finder.find("fps") {
                    self.fps_display = Some(entity);
                }
            });
        }

        // it is important that the 'paused' field is actually pausing your game.
        // Make sure to also pause your running systems.
        if !self.paused {
            let mut ui_text = world.write_storage::<UiText>();

            if let Some(fps_display) = self.fps_display.and_then(|entity| ui_text.get_mut(entity)) {
                if world.read_resource::<Time>().frame_number() % 20 == 0 && !self.paused {
                    let fps = world.read_resource::<FpsCounter>().sampled_fps();
                    fps_display.text = format!("FPS: {:.*}", 2, fps);
                }
            }
        }

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

    lazy_update.insert(fire_entity, Removal::new(0 as u32));
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
