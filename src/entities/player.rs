use amethyst::{
    assets::Handle,
    core::transform::Transform,
    ecs::prelude::{Entity, World},
    prelude::*,
    renderer::{palette::Srgba, resources::Tint, SpriteRender, SpriteSheet, Transparent},
    utils::removal::Removal,
};

use crate::entities::ui::PlayerStatusText;
use amethyst::core::math::Vector3;
use std::f32::consts::PI;

use crate::components::{
    build_named_weapon_from_world, get_weapon_icon, Player, PlayerWeaponIcon, Vehicle, WeaponArray, Weapon, WeaponNames,
    VehicleMovementType, VehicleTypes, get_vehicle_sprites,
};
use crate::resources::{GameModeSetup, GameModes, GameWeaponSetup, WeaponFireResource};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};

pub fn intialize_player(
    world: &mut World,
    sprite_sheet_handle: Handle<SpriteSheet>,
    player_index: usize,
    weapon_fire_resource: WeaponFireResource,
    team: i32,
    is_bot: bool,
    player_status_text: PlayerStatusText,
    vehicle_type: VehicleTypes,
    heal_pulse_amount: f32,
    heal_pulse_rate: f32,
    max_health: f32,
    max_armor: f32,
    max_shield: f32,
    engine_force: f32,
    engine_weight: f32,
    max_velocity: f32,
    vehicle_movement_type: VehicleMovementType,
    vehicle_width: f32,
    vehicle_height: f32,
    vehicle_sprite_scalar: f32,
) -> Entity {
    let game_mode;
    let weapon_name;
    {
        let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();

        if let Some(game_mode_setup) = fetched_game_mode_setup {
            game_mode = game_mode_setup.game_mode.clone();
        } else {
            game_mode = GameModes::ClassicGunGame;
        }

        let fetched_game_weapon_setup = world.try_fetch::<GameWeaponSetup>();

        if let Some(game_weapon_setup) = fetched_game_weapon_setup {
            weapon_name = game_weapon_setup.starter_weapon.clone();
        } else {
            weapon_name = WeaponNames::LaserDoubleGimballed;
        }
    }

    let mut vehicle_transform = Transform::default();

    let spacing_factor = 5.0;

    let height = ARENA_HEIGHT + UI_HEIGHT;

    let starting_rotation;
    let starting_x;
    let starting_y;

    if game_mode == GameModes::Race {
        let (x, y) = match player_index {
            0 => (ARENA_WIDTH - 70.0, height / 2.0 - 14.0),
            1 => (ARENA_WIDTH - 50.0, height / 2.0 - 14.0),
            2 => (ARENA_WIDTH - 30.0, height / 2.0 - 14.0),
            3 => (ARENA_WIDTH - 10.0, height / 2.0 - 14.0),
            _ => (ARENA_WIDTH - 40.0, height / 2.0 - 14.0),
        };

        starting_rotation = 0.0;
        starting_x = x;
        starting_y = y;
    } else {
        let (rotation, x, y) = match player_index {
            0 => (
                -PI / 4.0,
                ARENA_WIDTH / spacing_factor,
                height / spacing_factor,
            ),
            1 => (
                PI / 2.0 + PI / 4.0,
                ARENA_WIDTH - (ARENA_WIDTH / spacing_factor),
                height - (height / spacing_factor),
            ),
            2 => (
                PI + PI / 4.0,
                ARENA_WIDTH / spacing_factor,
                height - (height / spacing_factor),
            ),
            3 => (
                PI / 2.0 - PI / 4.0,
                ARENA_WIDTH - (ARENA_WIDTH / spacing_factor),
                height / spacing_factor,
            ),
            _ => (
                -PI / 4.0,
                ARENA_WIDTH / spacing_factor,
                height / spacing_factor,
            ),
        };

        starting_rotation = rotation;
        starting_x = x;
        starting_y = y;
    }

    vehicle_transform.set_rotation_2d(starting_rotation as f32);
    vehicle_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.0);
    vehicle_transform.set_scale(Vector3::new(1./vehicle_sprite_scalar, 1./vehicle_sprite_scalar, 0.0));


    let (vehicle_sprite_number, shield_sprite_number, armor_sprite_number) = get_vehicle_sprites(vehicle_type);

    let vehicle_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: vehicle_sprite_number + player_index,
    };

    let weapon_stats = build_named_weapon_from_world(weapon_name.clone(), world);

    //Create Health Entity
    let mut health_transform = Transform::default();
    health_transform.set_rotation_2d(starting_rotation as f32);
    health_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.3);
    //health_transform.set_scale(Vector3::new(1./vehicle_sprite_scalar, 1./vehicle_sprite_scalar, 0.0));

    let health_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 21,
    };

    // White shows the sprite as normal.
    // You can change the color at any point to modify the sprite's tint.
    let health_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 0.0));

    let health_entity = world
        .create_entity()
        .with(Removal::new(0 as u32))
        .with(health_transform)
        .with(health_sprite_render)
        .with(Transparent)
        .with(health_tint)
        .build();

    //Create Repair Lines Entity
    let mut repair_transform = Transform::default();
    repair_transform.set_rotation_2d(starting_rotation as f32);
    repair_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.4);
    //repair_transform.set_scale(Vector3::new(1./vehicle_sprite_scalar, 1./vehicle_sprite_scalar, 0.0));

    let repair_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 22,
    };

    // White shows the sprite as normal.
    // You can change the color at any point to modify the sprite's tint.
    let repair_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 0.0));

    let repair_entity = world
        .create_entity()
        .with(Removal::new(0 as u32))
        .with(repair_transform)
        .with(repair_sprite_render)
        .with(Transparent)
        .with(repair_tint)
        .build();

    //Create Armor Entity
    let mut armor_transform = Transform::default();
    armor_transform.set_rotation_2d(starting_rotation as f32);
    armor_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.2);
    armor_transform.set_scale(Vector3::new(1./vehicle_sprite_scalar, 1./vehicle_sprite_scalar, 0.0));

    let armor_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: armor_sprite_number,
    };

    // White shows the sprite as normal.
    // You can change the color at any point to modify the sprite's tint.
    let armor_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

    let armor_entity = world
        .create_entity()
        .with(Removal::new(0 as u32))
        .with(armor_transform)
        .with(armor_sprite_render)
        .with(Transparent)
        .with(armor_tint)
        .build();

    //Create Shield Entity
    let mut shield_transform = Transform::default();
    shield_transform.set_rotation_2d(starting_rotation as f32);
    shield_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.1);
    shield_transform.set_scale(Vector3::new(1./vehicle_sprite_scalar, 1./vehicle_sprite_scalar, 0.0));

    let shield_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: shield_sprite_number,
    };

    // White shows the sprite as normal.
    // You can change the color at any point to modify the sprite's tint.
    let shield_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

    let shield_entity = world
        .create_entity()
        .with(Removal::new(0 as u32))
        .with(shield_transform)
        .with(shield_sprite_render)
        .with(Transparent)
        .with(shield_tint)
        .build();



    //UI vehicle icons
    let x = 20.;
    let y = UI_HEIGHT - 10.;
    let dx = 32.;
    let dx2 = 4.;
    {
        let mut icon_transform = Transform::default();

        let starting_x = match player_index {
            0 => (x),
            1 => (x + 3.0 * dx + dx2),
            2 => (x + 6.0 * dx + 2.0 * dx2),
            3 => (x + 9.0 * dx + 3.0 * dx2),
            _ => (0.0),
        };

        icon_transform.set_rotation_2d(-PI / 2.0);
        icon_transform.set_translation_xyz(starting_x as f32, y, 0.4);

        let vehicle_icon_scale = 1.0 / vehicle_sprite_scalar;
        icon_transform.set_scale(Vector3::new(vehicle_icon_scale, vehicle_icon_scale, 0.0));

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(icon_transform)
            .with(vehicle_sprite_render.clone())
            .build();
    }

    //UI initial weapon icon
    let x = 5.;
    let y = UI_HEIGHT - 10.;
    let dx = 32.;
    let dx2 = 4.;

    let weapon_icon_dx = 70.0;

    let (icon_scale, weapon_sprite) =
        get_weapon_icon(player_index, weapon_stats, &weapon_fire_resource);

    let mut icon_weapon_transform = Transform::default();

    let starting_x = match player_index {
        0 => (x),
        1 => (x + 3.0 * dx + dx2),
        2 => (x + 6.0 * dx + 2.0 * dx2),
        3 => (x + 9.0 * dx + 3.0 * dx2),
        _ => (0.0),
    };

    icon_weapon_transform.set_translation_xyz((starting_x + weapon_icon_dx) as f32, y, 0.4);
    icon_weapon_transform.set_rotation_2d(-PI / 2.0);
    icon_weapon_transform.set_scale(Vector3::new(icon_scale, icon_scale, 0.0));

    // White shows the sprite as normal.
    // You can change the color at any point to modify the sprite's tint.
    let icon_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

    let weapon_icon = world
        .create_entity()
        .with(Removal::new(0 as u32))
        .with(PlayerWeaponIcon::new(
            player_index,
            weapon_stats.weapon_type,
        ))
        .with(weapon_sprite)
        .with(icon_weapon_transform)
        .with(icon_tint)
        .with(Transparent)
        .build();

    let weapon = Weapon::new(weapon_name, weapon_icon, weapon_stats);

    //Create actual Player with Vehicle and Weapon
    world
        .create_entity()
        .with(Removal::new(0 as u32))
        .with(vehicle_transform)
        .with(vehicle_sprite_render)
        .with(Vehicle::new(
            player_status_text,
            health_entity,
            armor_entity,
            shield_entity,
            repair_entity,
            max_shield,
            max_armor,
            max_health,
            heal_pulse_amount,
            heal_pulse_rate,
            engine_force,
            engine_weight,
            max_velocity,
            weapon_stats.weight,
            vehicle_movement_type,
            vehicle_width,
            vehicle_height,
        ))
        .with(WeaponArray {
            weapons: [Some(weapon), None, None, None],
        })
        .with(Player::new(player_index, team, is_bot))
        .build()
}
