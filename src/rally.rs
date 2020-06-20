use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::Time,
    ecs::prelude::{Dispatcher, DispatcherBuilder, Entity},
    input::{is_close_requested, is_key_down},
    prelude::*,
    renderer::{
        debug_drawing::{DebugLines, DebugLinesComponent, DebugLinesParams},
        ImageFormat, SpriteSheet, SpriteSheetFormat, Texture
    },
    ui::{UiCreator, UiFinder, UiText, UiTransform},
    utils::{
        fps_counter::FpsCounter,
        removal::{exec_removal, Removal},
    },
    winit::VirtualKeyCode,
};

use crate::pause::PauseMenuState;
use crate::score_screen::ScoreScreen;

use crate::resources::{initialize_weapon_fire_resource, GameModeSetup, GameScore, 
    GameTeamSetup, WeaponFireResource, GameVehicleSetup, ArenaNavMesh, ArenaInvertedNavMesh,
};

use crate::entities::{
    initialize_arena_walls, initialize_camera, initialize_camera_to_player, initialize_timer_ui,
    connect_players_to_ui, intialize_player, PlayerStatusText,
};

use crate::components::{
    build_weapon_store, Armor, Health, Hitbox, Player, PlayerWeaponIcon, Repair, Shield, Vehicle,
    WeaponArray, WeaponFire, Particles, VehicleMovementType, VehicleTypes,
};

use crate::systems::{
    CollisionVehToVehSystem, CollisionVehicleWeaponFireSystem, MoveWeaponFireSystem,
    VehicleMoveSystem, VehicleShieldArmorHealthSystem, VehicleStatusSystem, VehicleTrackingSystem,
    VehicleWeaponsSystem, MoveParticlesSystem, DebugLinesSystem,
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
    player_ui_initialized: bool,

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
        self.player_ui_initialized = false;

        let world = &mut data.world;

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/gameplay.ron", ())));

        world.register::<UiText>();
        world.register::<UiTransform>();

        world.register::<Armor>();
        world.register::<Health>();
        world.register::<Hitbox>();
        world.register::<Player>();
        world.register::<Repair>();
        world.register::<Shield>();
        world.register::<Vehicle>();
        world.register::<WeaponArray>();
        world.register::<WeaponFire>();
        world.register::<Particles>();

        world.register::<PlayerWeaponIcon>();



        
        // Setup debug lines as a resource
        world.insert(DebugLines::new());
        // Configure width of lines. Optional step
        world.insert(DebugLinesParams { line_width: 2.0 });

        // Setup debug lines as a component and add lines to render axis&grid
        let debug_lines_component = DebugLinesComponent::new();

        world
            .create_entity()
            .with(debug_lines_component)
            .build();




        world.register::<Removal<u32>>();

        self.sprite_sheet_handle.replace(load_sprite_sheet(
            world,
            "texture/rally_spritesheet.png".to_string(),
            "texture/rally_spritesheet.ron".to_string(),
        ));
        self.texture_sheet_handle.replace(load_sprite_sheet(
            world,
            "texture/rally_texture_sheet.png".to_string(),
            "texture/rally_texture_sheet.ron".to_string(),
        ));

        let weapon_fire_resource: WeaponFireResource =
            initialize_weapon_fire_resource(world, self.sprite_sheet_handle.clone().unwrap());

        let weapon_store = build_weapon_store(world);

        initialize_timer_ui(world);


        world.insert(ArenaNavMesh {
            vertices: Vec::new(),
            triangles: Vec::new(),
        });

        world.insert(ArenaInvertedNavMesh {
            vertices: Vec::new(),
            triangles: Vec::new(),
        });

        

        initialize_arena_walls(
            world,
            self.sprite_sheet_handle.clone().unwrap(),
            self.texture_sheet_handle.clone().unwrap(),
        );


        let max_players;
        let bot_players;
        {
            let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();

            if let Some(game_mode_setup) = fetched_game_mode_setup {
                max_players = game_mode_setup.max_players;
                bot_players = game_mode_setup.bot_players;
            } else {
                max_players = 4;
                bot_players = 3;
            }
        }

        let player_to_team;
        {
            let fetched_game_team_setup = world.try_fetch::<GameTeamSetup>();

            if let Some(game_team_setup) = fetched_game_team_setup {
                player_to_team = game_team_setup.teams.clone();
            } else {
                player_to_team = [0, 1, 2, 3];
            }
        }


        
        let player_status_text = PlayerStatusText {
            shield: None,
            armor: None,
            health: None,
            points: None,
            lives: None
        };

        for player_index in 0..max_players {
            let vehicle_type: VehicleTypes;

            let max_health: f32;
            let max_armor: f32;
            let max_shield: f32;

            let engine_force: f32;
            let engine_weight: f32;

            let vehicle_width: f32;
            let vehicle_height: f32;
            let vehicle_sprite_scalar: f32;

            let max_velocity: f32;

            let vehicle_movement_type: VehicleMovementType;

            {
                let fetched_game_vehicle_setup = world.try_fetch::<GameVehicleSetup>();

                if let Some(game_vehicle_setup) = fetched_game_vehicle_setup {
                    vehicle_type = game_vehicle_setup.stats[player_index].vehicle_type;

                    max_health = game_vehicle_setup.stats[player_index].max_health;
                    max_armor = game_vehicle_setup.stats[player_index].max_armor;
                    max_shield = game_vehicle_setup.stats[player_index].max_shield;

                    engine_force = game_vehicle_setup.stats[player_index].engine_force;
                    engine_weight = game_vehicle_setup.stats[player_index].engine_weight;

                    vehicle_width = game_vehicle_setup.stats[player_index].width;
                    vehicle_height = game_vehicle_setup.stats[player_index].height;
                    vehicle_sprite_scalar = game_vehicle_setup.stats[player_index].sprite_scalar;

                    max_velocity = game_vehicle_setup.stats[player_index].max_velocity;

                    vehicle_movement_type = VehicleMovementType::Hover;
                }
                else {
                    vehicle_type = VehicleTypes::MediumCombat;

                    max_health = 100.0;
                    max_armor = 100.0;
                    max_shield = 100.0;

                    engine_force = 100.0;
                    let engine_efficiency = 1.0;
                    engine_weight = engine_force / engine_efficiency * 20. / 100.;

                    vehicle_width = 7.0;
                    vehicle_height = 12.0;
                    vehicle_sprite_scalar = 1.0;

                    max_velocity = 1.0;

                    vehicle_movement_type = VehicleMovementType::Hover;
                }
            }

            let is_bot = player_index >= max_players - bot_players;
            

            let player = intialize_player(
                world,
                self.sprite_sheet_handle.clone().unwrap(),
                player_index,
                weapon_fire_resource.clone(),
                weapon_store.clone(),
                player_to_team[player_index],
                is_bot,
                player_status_text.clone(),
                vehicle_type.clone(),
                max_health,
                max_armor,
                max_shield,
                engine_force,
                engine_weight,
                max_velocity,
                vehicle_movement_type,
                vehicle_width,
                vehicle_height,
                vehicle_sprite_scalar,
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
        dispatcher_builder.add(VehicleTrackingSystem, "vehicle_tracking_system", &[]);
        dispatcher_builder.add(VehicleMoveSystem::default(), "vehicle_move_system", &[]);

        dispatcher_builder.add(VehicleWeaponsSystem, "vehicle_weapons_system", &[]);
        dispatcher_builder.add(
            CollisionVehicleWeaponFireSystem::default(),
            "collision_vehicle_weapon_fire_system",
            &["vehicle_weapons_system"],
        );
        dispatcher_builder.add(
            MoveWeaponFireSystem,
            "move_weapon_fire_system",
            &["collision_vehicle_weapon_fire_system"],
        );

        dispatcher_builder.add(
            CollisionVehToVehSystem,
            "collision_vehicle_vehicle_system",
            &["vehicle_move_system"],
        );
        
        dispatcher_builder.add(
            VehicleShieldArmorHealthSystem,
            "vehicle_shield_armor_health_system",
            &[],
        );
        dispatcher_builder.add(VehicleStatusSystem::default(), "vehicle_status_system", &[]);

        dispatcher_builder.add(MoveParticlesSystem, "move_particles_system", &[]);



        dispatcher_builder.add(DebugLinesSystem, "debug_lines_system", &[]);



        // Build and setup the `Dispatcher`.
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);
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


        let fetched_game_score = data.world.try_fetch::<GameScore>();

        if let Some(game_score) = fetched_game_score {
            if !game_score.game_ended {
                exec_removal(&data.world.entities(), &data.world.read_storage(), 0 as u32);
            }
        }
        else {
            exec_removal(&data.world.entities(), &data.world.read_storage(), 0 as u32);
        }

        self.player_ui_initialized = false;

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


        if !self.player_ui_initialized {
            let connected_success = connect_players_to_ui(world);

            if connected_success {
                self.player_ui_initialized = true;
            }
        }


        let fetched_game_score = world.try_fetch::<GameScore>();

        if let Some(game_score) = fetched_game_score {
            if game_score.game_ended {
                return Trans::Switch(Box::new(ScoreScreen::default()));
            }
        }

        Trans::None
    }
}

pub fn load_sprite_sheet(world: &mut World, storage: String, store: String) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `texture_handle` is a cloneable reference to the texture
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(storage, ImageFormat::default(), (), &texture_storage)
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
