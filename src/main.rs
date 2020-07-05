use amethyst::{
    assets::HotReloadBundle,
    audio::AudioBundle,
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow, RenderDebugLines},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::{application_root_dir, fps_counter::FpsCounterBundle},
};

mod credits;
mod menu;
mod pause;
mod rally;
mod welcome;
mod custom_vehicles;
mod custom_weapons;
mod custom_arena;
mod score_screen;

mod audio;
mod components;
mod entities;
mod resources;
mod systems;

use crate::rally::{MovementBindingTypes, MP_BINDINGS, CONTROLLER_BINDINGS};
use crate::welcome::WelcomeScreen;

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
    if CONTROLLER_BINDINGS {
        binding_path = config_dir.join("bindings_controller.ron");
        input_bundle =
            InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;
    }
    else {
        binding_path = config_dir.join("bindings.ron");
        input_bundle =
            InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;
    }

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
                        .with_clear([0.03, 0.03, 0.03, 1.0])
                        //.with_clear([0.14, 0.14, 0.13, 1.0]), //background color R,G,B
                )
                // RenderFlat2D plugin is used to render entities with a `SpriteRender` component.
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default())
                .with_plugin(RenderDebugLines::default())
        )?;

    let mut game = Application::new(assets_dir, WelcomeScreen::default(), game_data)?;

    game.run();

    Ok(())
}
