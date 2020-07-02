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

use crate::resources::{initialize_weapon_fire_resource, 
    GameModeSetup, GameScore, 
    GameTeamSetup, WeaponFireResource, GameVehicleSetup, 
    ArenaNavMesh, ArenaNavMeshFinal,
};

use crate::entities::{
    intialize_arena, initialize_camera, initialize_camera_to_player, initialize_timer_ui,
    connect_players_to_ui, intialize_player, PlayerStatusText, 
};

use crate::components::{
    ArenaElement, Hitbox, 
    Player, PlayerWeaponIcon, Repair, Shield, Armor, Health, Vehicle,
    WeaponArray, WeaponFire, Particles, get_none_vehicle,
    ArenaNames, ArenaStoreResource, ArenaProperties,
};

use crate::systems::{
    CollisionVehToVehSystem, CollisionWeaponFireHitboxSystem, MoveWeaponFireSystem,
    VehicleMoveSystem, VehicleShieldArmorHealthSystem, VehicleStatusSystem, VehicleTrackingSystem,
    VehicleWeaponsSystem, MoveParticlesSystem, PathingLinesSystem, CameraTrackingSystem,
};

pub const PLAYER_CAMERA: bool = false;
pub const DEBUG_LINES: bool = false;

//Damage at speed of 100
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
        world.register::<ArenaElement>();
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


        initialize_timer_ui(world);


        world.insert(ArenaNavMesh {
            vertices: Vec::new(),
            triangles: Vec::new(),
        });

        world.insert(ArenaNavMeshFinal {
            mesh: None,
        });


        intialize_arena(
            world,
            self.sprite_sheet_handle.clone().unwrap(),
            self.texture_sheet_handle.clone().unwrap(),
        );



        let max_players;
        let bot_players;
        let arena_name;
        {
            let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();

            if let Some(game_mode_setup) = fetched_game_mode_setup {
                max_players = game_mode_setup.max_players;
                bot_players = game_mode_setup.bot_players;
                arena_name = game_mode_setup.arena_name.clone();
            } else {
                max_players = 4;
                bot_players = 3;
                arena_name = ArenaNames::OpenEmptyMap;
            }
        }

        
        let arena_properties;
        {        
            let fetched_arena_store = world.try_fetch::<ArenaStoreResource>();

            if let Some(arena_store) = fetched_arena_store {
                arena_properties = match arena_store.properties.get(&arena_name) {
                    Some(arena_props_get) => (*arena_props_get).clone(),
                    _ => ArenaProperties::default(),
                };
            }
            else {
                arena_properties = ArenaProperties::default();
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
            let vehicle_stats;

            {
                let fetched_game_vehicle_setup = world.try_fetch::<GameVehicleSetup>();

                if let Some(game_vehicle_setup) = fetched_game_vehicle_setup {
                    vehicle_stats = game_vehicle_setup.stats[player_index].clone();
                }
                else {
                    vehicle_stats = get_none_vehicle();
                }
            }

            let is_bot = player_index >= max_players - bot_players;
            

            let player = intialize_player(
                world,
                self.sprite_sheet_handle.clone().unwrap(),
                player_index,
                weapon_fire_resource.clone(),
                player_to_team[player_index],
                is_bot,
                player_status_text.clone(),
                vehicle_stats,
            );

            if PLAYER_CAMERA && !is_bot {
                initialize_camera_to_player(world, &arena_properties, player);
            }
        }

        if !PLAYER_CAMERA {
            initialize_camera(world, &arena_properties);
        }

        // Create the `DispatcherBuilder` and register some `System`s that should only run for this `State`.
        let mut dispatcher_builder = DispatcherBuilder::new();
        dispatcher_builder.add(VehicleTrackingSystem, "vehicle_tracking_system", &[]);
        dispatcher_builder.add(VehicleMoveSystem::default(), "vehicle_move_system", &[]);

        dispatcher_builder.add(VehicleWeaponsSystem, "vehicle_weapons_system", &[]);
        dispatcher_builder.add(
            CollisionWeaponFireHitboxSystem::default(),
            "collision_weapon_fire_hitbox_system",
            &[],
        );
        dispatcher_builder.add(
            MoveWeaponFireSystem::default(),
            "move_weapon_fire_system",
            &[],
        );

        dispatcher_builder.add(
            CollisionVehToVehSystem,
            "collision_vehicle_vehicle_system",
            &[],
        );
        
        dispatcher_builder.add(
            VehicleShieldArmorHealthSystem,
            "vehicle_shield_armor_health_system",
            &[],
        );
        dispatcher_builder.add(VehicleStatusSystem::default(), "vehicle_status_system", &[]);

        dispatcher_builder.add(MoveParticlesSystem, "move_particles_system", &[]);

        dispatcher_builder.add(PathingLinesSystem::default(), "pathing_lines_system", &[]);

        dispatcher_builder.add(CameraTrackingSystem::default(), "camera_tracking_system", &[]);
        

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
