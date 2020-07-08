use amethyst::{
    assets::HotReloadBundle,
    audio::AudioBundle,
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderDebugLines, RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::{application_dir, application_root_dir, fps_counter::FpsCounterBundle},
};
use std::fs::File;
use std::path::PathBuf;

mod credits;
mod custom_arena;
mod custom_vehicles;
mod custom_weapons;
mod menu;
mod pause;
mod rally;
mod score_screen;
mod welcome;

mod audio;
mod components;
mod entities;
mod resources;
mod systems;

use crate::welcome::WelcomeScreen;
use serde::de::DeserializeOwned;

fn load_ron_asset<T: DeserializeOwned>(path: &[&str]) -> T {
    let mut path_buf = PathBuf::from("assets");
    path_buf.extend(path);
    let path = application_dir(path_buf).expect("Failed to find application directory");

    let file = File::open(&path).expect(&format!("Failed to open file: {:?}", path));

    ron::de::from_reader(file).expect("Failed to load config")
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");

    let config_dir = app_root.join("config");
    let display_config_path = config_dir.join("display.ron");

    let binding_path;
    let input_bundle;
    // if MP_BINDINGS {
    //     binding_path_mp = config_dir.join("bindings_mp.ron");
    //     input_bundle_mp = InputBundle::<MovementBindingTypes>::new()
    //         .with_bindings_from_file(binding_path)?;
    // }

    binding_path = config_dir.join("bindings_controller.ron");
    input_bundle = InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;
    // else {
    //     binding_path = config_dir.join("bindings.ron");
    //     input_bundle =
    //         InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;
    // }

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        //.with_bundle(input_bundle_mp)?
        .with_bundle(input_bundle)?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_bundle(HotReloadBundle::default())?
        .with_bundle(AudioBundle::default())?
        // .with_system_desc(
        //     crate::events::UiEventHandlerSystemDesc::default(),
        //     "ui_event_handler",
        //     &[],
        // )
        .with_bundle(FpsCounterBundle)?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                // The RenderToWindow plugin provides all the scaffolding for opening a window and drawing on it
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.03, 0.03, 0.03, 1.0]), //.with_clear([0.14, 0.14, 0.13, 1.0]), //background color R,G,B
                )
                // RenderFlat2D plugin is used to render entities with a `SpriteRender` component.
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default())
                .with_plugin(RenderDebugLines::default()),
        )?;

    let mut game = Application::new(assets_dir, WelcomeScreen::default(), game_data)?;

    game.run();

    Ok(())
}
