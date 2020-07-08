use amethyst::{
    assets::Handle,
    core::transform::Transform,
    ecs::prelude::{Entity, World},
    prelude::*,
    renderer::{palette::Srgba, resources::Tint, SpriteRender, SpriteSheet, Transparent},
    ui::{Anchor, UiImage, UiTransform},
    utils::removal::Removal,
};

use crate::entities::ui::PlayerStatusText;
use amethyst::core::math::Vector3;
use std::f32::consts::PI;

use crate::components::{
    build_named_weapon_from_world, get_vehicle_sprites, get_weapon_icon, get_weapon_width_height,
    ArenaNames, ArenaProperties, ArenaStoreResource, Player, PlayerWeaponIcon, Vehicle,
    VehicleStats, Weapon, WeaponArray, WeaponInstall, WeaponNameInstall,
};
use crate::resources::{
    GameModeSetup, GameWeaponSelectionMode, GameWeaponSetup, WeaponFireResource,
};

pub fn intialize_player(
    world: &mut World,
    sprite_sheet_handle: Handle<SpriteSheet>,
    player_index: usize,
    weapon_fire_resource: WeaponFireResource,
    team: i32,
    is_bot: bool,
    player_status_text: PlayerStatusText,
    vehicle_stats: VehicleStats,
) -> Entity {
    let mut weapon_named_installs: Vec<WeaponNameInstall> = Vec::new();
    {
        let fetched_game_weapon_setup = world.try_fetch::<GameWeaponSetup>();
        if let Some(game_weapon_setup) = fetched_game_weapon_setup {
            if game_weapon_setup.mode == GameWeaponSelectionMode::StarterAndPickup
                || game_weapon_setup.mode == GameWeaponSelectionMode::CustomStarterAndPickup
            {
                weapon_named_installs.push(WeaponNameInstall {
                    firing_group: 0,
                    weapon_name: game_weapon_setup.starter_weapon.clone(),
                    ammo: None,
                    mounted_angle: None,
                    x_offset: None,
                    y_offset: None,
                });
            } else if game_weapon_setup.mode == GameWeaponSelectionMode::GunGameForward {
                weapon_named_installs.push(WeaponNameInstall {
                    firing_group: 0,
                    weapon_name: game_weapon_setup.starter_weapon.clone(),
                    ammo: None,
                    mounted_angle: None,
                    x_offset: None,
                    y_offset: None,
                });
            } else if game_weapon_setup.mode == GameWeaponSelectionMode::GunGameReverse {
                weapon_named_installs.push(WeaponNameInstall {
                    firing_group: 0,
                    weapon_name: game_weapon_setup.starter_weapon.clone(),
                    ammo: None,
                    mounted_angle: None,
                    x_offset: None,
                    y_offset: None,
                });
            } else if game_weapon_setup.mode == GameWeaponSelectionMode::GunGameRandom {
                weapon_named_installs.push(WeaponNameInstall {
                    firing_group: 0,
                    weapon_name: game_weapon_setup.starter_weapon.clone(),
                    ammo: None,
                    mounted_angle: None,
                    x_offset: None,
                    y_offset: None,
                });
            } else if game_weapon_setup.mode == GameWeaponSelectionMode::FullCustom {
                for weapon_name_install in vehicle_stats.default_weapons.iter() {
                    weapon_named_installs.push(WeaponNameInstall {
                        firing_group: weapon_name_install.firing_group,
                        weapon_name: weapon_name_install.weapon_name,
                        ammo: weapon_name_install.ammo,
                        mounted_angle: weapon_name_install.mounted_angle,
                        x_offset: weapon_name_install.x_offset,
                        y_offset: weapon_name_install.y_offset,
                    });
                }
            } else if game_weapon_setup.mode == GameWeaponSelectionMode::VehiclePreset {
                for weapon_name_install in vehicle_stats.default_weapons.iter() {
                    weapon_named_installs.push(WeaponNameInstall {
                        firing_group: weapon_name_install.firing_group,
                        weapon_name: weapon_name_install.weapon_name,
                        ammo: weapon_name_install.ammo,
                        mounted_angle: weapon_name_install.mounted_angle,
                        x_offset: weapon_name_install.x_offset,
                        y_offset: weapon_name_install.y_offset,
                    });
                }
            }
        }
    }

    //Get Arena's properties
    let arena_name;
    {
        let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();

        if let Some(game_mode_setup) = fetched_game_mode_setup {
            arena_name = game_mode_setup.arena_name.clone();
        } else {
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
        } else {
            arena_properties = ArenaProperties::default();
        }
    }

    let player_spawn = arena_properties.player_spawn_points[player_index];

    let mut vehicle_transform = Transform::default();
    vehicle_transform.set_rotation_2d(player_spawn.rotation / 180.0 * PI);
    vehicle_transform.set_translation_xyz(player_spawn.x, player_spawn.y, 0.0);
    vehicle_transform.set_scale(Vector3::new(
        1.0 / vehicle_stats.sprite_scalar,
        1.0 / vehicle_stats.sprite_scalar,
        0.0,
    ));

    let (vehicle_sprite_number, shield_sprite_number, armor_sprite_number) =
        get_vehicle_sprites(&world, vehicle_stats.vehicle_type);

    let vehicle_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: vehicle_sprite_number + player_index,
    };

    //Create Health Entity
    let mut health_transform = Transform::default();
    health_transform.set_rotation_2d(player_spawn.rotation as f32);
    health_transform.set_translation_xyz(player_spawn.x as f32, player_spawn.y as f32, 0.3);
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
    repair_transform.set_rotation_2d(player_spawn.rotation as f32);
    repair_transform.set_translation_xyz(player_spawn.x as f32, player_spawn.y as f32, 0.4);
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
    armor_transform.set_rotation_2d(player_spawn.rotation as f32);
    armor_transform.set_translation_xyz(player_spawn.x as f32, player_spawn.y as f32, 0.2);
    armor_transform.set_scale(Vector3::new(
        1.0 / vehicle_stats.sprite_scalar,
        1.0 / vehicle_stats.sprite_scalar,
        0.0,
    ));

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
    shield_transform.set_rotation_2d(player_spawn.rotation as f32);
    shield_transform.set_translation_xyz(player_spawn.x as f32, player_spawn.y as f32, 0.1);
    shield_transform.set_scale(Vector3::new(
        1.0 / vehicle_stats.sprite_scalar,
        1.0 / vehicle_stats.sprite_scalar,
        0.0,
    ));

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
    let x = -450.;
    let y = 45.;
    let dx = 250.;
    {
        let starting_x = match player_index {
            0 => (x),
            1 => (x + dx),
            2 => (x + 2.0 * dx),
            3 => (x + 3.0 * dx),
            _ => (0.0),
        };

        let icon_transform = UiTransform::new(
            "PVehicle".to_string(),
            Anchor::BottomMiddle,
            Anchor::BottomMiddle,
            starting_x,
            y,
            0.2,
            vehicle_stats.width * 3.0,
            vehicle_stats.height * 3.0,
        );

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(icon_transform)
            .with(UiImage::Sprite(vehicle_sprite_render.clone()))
            .build();
    }

    let mut total_weapon_weight = 0.0;
    let mut installed_weapons: Vec<WeaponInstall> = Vec::new();

    for (weapon_array_id, weapon_name_install) in weapon_named_installs.iter().enumerate() {
        let weapon_stats =
            build_named_weapon_from_world(weapon_name_install.weapon_name.clone(), world);

        //UI initial weapon icon
        let x = -320. + (weapon_array_id as f32) * 30.0;

        let (icon_scale, weapon_sprite) = get_weapon_icon(
            Some(player_index),
            weapon_stats.weapon_fire_type.clone(),
            &weapon_fire_resource,
        );

        let starting_x = match player_index {
            0 => (x),
            1 => (x + dx),
            2 => (x + 2.0 * dx),
            3 => (x + 3.0 * dx),
            _ => (0.0),
        };

        let (weapon_width, weapon_height) =
            get_weapon_width_height(weapon_stats.weapon_fire_type.clone());

        let icon_weapon_transform = UiTransform::new(
            "PWeaponIcon".to_string(),
            Anchor::BottomMiddle,
            Anchor::BottomMiddle,
            starting_x,
            y,
            0.2,
            weapon_width * icon_scale,
            weapon_height * icon_scale,
        );

        // White shows the sprite as normal.
        // You can change the color at any point to modify the sprite's tint.
        let icon_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

        let weapon_icon = world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(PlayerWeaponIcon::new(
                player_index,
                0,
                weapon_stats.weapon_fire_type,
            ))
            .with(UiImage::Sprite(weapon_sprite))
            .with(icon_weapon_transform)
            .with(icon_tint)
            .with(Transparent)
            .build();

        let weapon = Weapon::new(
            weapon_name_install.weapon_name,
            weapon_icon,
            weapon_stats.clone(),
            weapon_name_install.ammo,
        );

        installed_weapons.push(WeaponInstall {
            weapon,
            firing_group: weapon_name_install.firing_group,
            ammo: weapon_name_install.ammo,
            mounted_angle: weapon_name_install.mounted_angle,
            x_offset: weapon_name_install.x_offset,
            y_offset: weapon_name_install.y_offset,
        });

        total_weapon_weight += weapon_stats.weight;
    }

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
            vehicle_stats,
            total_weapon_weight,
        ))
        .with(WeaponArray {
            installed: installed_weapons,
        })
        .with(Player::new(player_index, team, is_bot))
        .build()
}
