use amethyst::{
    assets::Handle,
    core::transform::Transform,
    ecs::prelude::{Entity, World},
    prelude::*,
    renderer::{palette::Srgba, resources::Tint, SpriteRender, SpriteSheet, Transparent},
    utils::removal::Removal,
    ui::{UiTransform, UiImage, Anchor},
};

use crate::entities::ui::PlayerStatusText;
use amethyst::core::math::Vector3;
use std::f32::consts::PI;

use crate::components::{
    Player, PlayerWeaponIcon, 
    build_named_weapon_from_world, get_weapon_icon, WeaponArray, Weapon, WeaponNames,
    Vehicle, VehicleMovementType, VehicleTypes, get_vehicle_sprites, get_weapon_width_height,
    ArenaStoreResource, ArenaNames, ArenaProperties,
};
use crate::resources::{GameModeSetup, GameWeaponSetup, WeaponFireResource};


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
    let weapon_name;
    {
        let fetched_game_weapon_setup = world.try_fetch::<GameWeaponSetup>();

        if let Some(game_weapon_setup) = fetched_game_weapon_setup {
            weapon_name = game_weapon_setup.starter_weapon.clone();
        } else {
            weapon_name = WeaponNames::LaserDoubleGimballed;
        }
    }

    let mut vehicle_transform = Transform::default();


    
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
        }
        else {
            arena_properties = ArenaProperties::default();
        }
    }

    
    
    let player_spawn = arena_properties.player_spawn_points[player_index];

    vehicle_transform.set_rotation_2d(player_spawn.rotation/180.0*PI);
    vehicle_transform.set_translation_xyz(player_spawn.x, player_spawn.y, 0.0);
    vehicle_transform.set_scale(Vector3::new(1./vehicle_sprite_scalar, 1./vehicle_sprite_scalar, 0.0));


    let (vehicle_sprite_number, shield_sprite_number, armor_sprite_number) = get_vehicle_sprites(
        &world, vehicle_type
    );

    let vehicle_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: vehicle_sprite_number + player_index,
    };

    let weapon_stats = build_named_weapon_from_world(weapon_name.clone(), world);

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
    shield_transform.set_rotation_2d(player_spawn.rotation as f32);
    shield_transform.set_translation_xyz(player_spawn.x as f32, player_spawn.y as f32, 0.1);
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
    let x = -450.;
    let y = 45.;
    let dx = 250.;
    {
        let starting_x = match player_index {
            0 => (x),
            1 => (x + dx),
            2 => (x + 2.0*dx),
            3 => (x + 3.0*dx),
            _ => (0.0),
        };

        let icon_transform = UiTransform::new(
            "PVehicle".to_string(),
            Anchor::BottomMiddle,
            Anchor::BottomMiddle,
            starting_x,
            y,
            0.2,
            vehicle_width*3.0,
            vehicle_height*3.0,
        );

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(icon_transform)
            .with(UiImage::Sprite(vehicle_sprite_render.clone()))
            .build();
    }

    //UI initial weapon icon
    let x = -320.;

    let (icon_scale, weapon_sprite) =
        get_weapon_icon(player_index, weapon_stats.weapon_fire_type.clone(), &weapon_fire_resource);


    let starting_x = match player_index {
        0 => (x),
        1 => (x + dx),
        2 => (x + 2.0*dx),
        3 => (x + 3.0*dx),
        _ => (0.0),
    };

    let (weapon_width, weapon_height) = get_weapon_width_height(weapon_stats.weapon_fire_type.clone());

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


        
    let weapon = Weapon::new(weapon_name, weapon_icon, weapon_stats.clone());

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
