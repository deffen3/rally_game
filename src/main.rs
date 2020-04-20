use amethyst::{
    core::transform::TransformBundle,
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
};

use amethyst::input::{InputBundle};

use amethyst::audio::AudioBundle;

mod systems;

mod rally;

use crate::rally::{Rally, MovementBindingTypes};

mod audio;


fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
    let config_dir = app_root.join("config");
    let display_config_path = config_dir.join("display.ron");


    let binding_path = config_dir.join("bindings.ron");
    let input_bundle = InputBundle::<MovementBindingTypes>::new()
        .with_bindings_from_file(binding_path)?;


    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.44, 1.0]), //background color R,G,B
                )
                .with_plugin(RenderFlat2D::default()),
        )?
        .with_bundle(input_bundle)?
        //depends on input bundle system
        .with(systems::VehicleMoveSystem, "vehicle_move_system", &["input_system"])
        .with(systems::VehicleWeaponsSystem, "vehicle_weapons_system", &["input_system"])
        .with(systems::MoveWeaponFireSystem, "move_weapon_fire_system", &["vehicle_weapons_system"])
        .with(systems::CollisionVehToVehSystem, "collision_vehicle_vehicle_system", &["vehicle_move_system"])
        .with(systems::CollisionVehicleWeaponFireSystem::default(), "collision_vehicle_weapon_fire_system", &["vehicle_move_system"])
        .with_bundle(AudioBundle::default())?
        .with_bundle(TransformBundle::new())?;

    let mut game = Application::new(assets_dir, Rally::default(), game_data)?;
    game.run();

    Ok(())
}
