use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down, is_mouse_button_down},
    prelude::*,
    winit::{MouseButton, VirtualKeyCode},
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    assets::{AssetStorage, Handle, Loader},
    ui::{UiText, UiTransform, UiCreator},
};


use crate::audio::initialize_audio;

use crate::components::{
    Armor, Health, Hitbox, Player, Repair, Shield, Vehicle, Weapon, WeaponFire,
    PlayerWeaponIcon, WeaponNames,
};

use crate::entities::{initialize_arena_walls, initialize_camera, initialize_ui, intialize_player};

use crate::resources::{initialize_weapon_fire_resource, WeaponFireResource};

use crate::rally::{MAX_PLAYERS, BOT_PLAYERS};




#[derive(Default, Debug)]
pub struct WelcomeScreen {
    ui_handle: Option<Entity>,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>, // Load the spritesheet necessary to render the graphics.
    texture_sheet_handle: Option<Handle<SpriteSheet>>,
}

impl SimpleState for WelcomeScreen {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

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

        self.ui_handle =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/welcome.ron", ())));
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_mouse_button_down(&event, MouseButton::Left) {
                    log::info!("[Trans::Switch] Switching to MainMenu!");
                    Trans::Switch(Box::new(crate::menu::MainMenu::default()))
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(root_entity) = self.ui_handle {
            data.world
                .delete_entity(root_entity)
                .expect("Failed to remove WelcomeScreen");
        }

        self.ui_handle = None;
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