use amethyst::{
    assets::Handle,
    core::transform::Transform,
    ecs::prelude::World,
    prelude::*,
    renderer::{
        SpriteRender, SpriteSheet, Transparent,
        palette::Srgba,
        resources::Tint,
    },
};

use crate::entities::ui::PlayerStatusText;
use amethyst::core::math::Vector3;
use std::f32::consts::PI;

use crate::components::{
    build_named_weapon, Player, PlayerWeaponIcon, Vehicle, 
    Weapon, WeaponTypes, WeaponNames, get_mine_sprite,
};
use crate::resources::WeaponFireResource;

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};

pub fn intialize_player(
    world: &mut World,
    sprite_sheet_handle: Handle<SpriteSheet>,
    player_index: usize,
    weapon_name: WeaponNames,
    weapon_fire_resource: WeaponFireResource,
    is_bot: bool,
    player_status_text: PlayerStatusText,
) {
    let mut vehicle_transform = Transform::default();

    let spacing_factor = 5.0;

    let height = ARENA_HEIGHT + UI_HEIGHT;

    let (starting_rotation, starting_x, starting_y) = match player_index {
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

    vehicle_transform.set_rotation_2d(starting_rotation as f32);
    vehicle_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.0);

    let vehicle_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: player_index,
    };

    let weapon_stats = build_named_weapon(weapon_name.clone());



    //Create Health Entity
    let mut health_transform = Transform::default();
    health_transform.set_rotation_2d(starting_rotation as f32);
    health_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.3);

    let health_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 21,
    };

    // White shows the sprite as normal.
    // You can change the color at any point to modify the sprite's tint.
    let health_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 0.0));

    let health_entity = world
        .create_entity()
        .with(health_transform)
        .with(health_sprite_render)
        .with(Transparent)
        .with(health_tint)
        .build();


    //Create Repair Lines Entity
    let mut repair_transform = Transform::default();
    repair_transform.set_rotation_2d(starting_rotation as f32);
    repair_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.4);

    let repair_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 22,
    };

    // White shows the sprite as normal.
    // You can change the color at any point to modify the sprite's tint.
    let repair_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 0.0));

    let repair_entity = world
        .create_entity()
        .with(repair_transform)
        .with(repair_sprite_render)
        .with(Transparent)
        .with(repair_tint)
        .build();


    //Create Armor Entity
    let mut armor_transform = Transform::default();
    armor_transform.set_rotation_2d(starting_rotation as f32);
    armor_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.2);

    let armor_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 20,
    };

    // White shows the sprite as normal.
    // You can change the color at any point to modify the sprite's tint.
    let armor_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

    let armor_entity = world
        .create_entity()
        .with(armor_transform)
        .with(armor_sprite_render)
        .with(Transparent)
        .with(armor_tint)
        .build();


    //Create Shield Entity
    let mut shield_transform = Transform::default();
    shield_transform.set_rotation_2d(starting_rotation as f32);
    shield_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.1);

    let shield_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 19,
    };

    // White shows the sprite as normal.
    // You can change the color at any point to modify the sprite's tint.
    let shield_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

    let shield_entity = world
        .create_entity()
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
        icon_transform.set_translation_xyz(starting_x as f32, y, 0.0);

        world
            .create_entity()
            .with(icon_transform)
            .with(vehicle_sprite_render.clone())
            .build();
    }

    //UI initial weapon icon
    let x = 15.;
    let y = UI_HEIGHT - 10.;
    let dx = 32.;
    let dx2 = 4.;

    let weapon_icon_dx = 70.0;

    let (icon_scale, mut weapon_sprite) = match weapon_stats.weapon_type {
        WeaponTypes::LaserDouble => (3.0, weapon_fire_resource.laser_double_sprite_render.clone()),
        WeaponTypes::LaserBeam => (1.0, weapon_fire_resource.laser_beam_sprite_render.clone()),
        WeaponTypes::LaserPulse => (3.0, weapon_fire_resource.laser_burst_sprite_render.clone()),
        WeaponTypes::ProjectileBurstFire => {
            (3.0, weapon_fire_resource.projectile_burst_render.clone())
        }
        WeaponTypes::ProjectileRapidFire => {
            (3.0, weapon_fire_resource.projectile_rapid_render.clone())
        }
        WeaponTypes::ProjectileCannonFire => (
            3.0,
            weapon_fire_resource.projectile_cannon_sprite_render.clone(),
        ),
        WeaponTypes::Missile => (2.0, weapon_fire_resource.missile_sprite_render.clone()),
        WeaponTypes::Rockets => (2.0, weapon_fire_resource.rockets_sprite_render.clone()),
        WeaponTypes::Mine => (2.0, weapon_fire_resource.mine_p1_sprite_render.clone()),
        WeaponTypes::LaserSword => (1.0, weapon_fire_resource.laser_sword_sprite_render.clone()),
    };

    if weapon_stats.weapon_type == WeaponTypes::Mine {
        weapon_sprite = get_mine_sprite(player_index, weapon_stats.shot_speed, &weapon_fire_resource);
    }

    let mut icon_weapon_transform = Transform::default();

    let starting_x = match player_index {
        0 => (x),
        1 => (x + 3.0 * dx + dx2),
        2 => (x + 6.0 * dx + 2.0 * dx2),
        3 => (x + 9.0 * dx + 3.0 * dx2),
        _ => (0.0),
    };

    icon_weapon_transform.set_translation_xyz((starting_x + weapon_icon_dx) as f32, y, 0.0);
    icon_weapon_transform.set_rotation_2d(-PI / 2.0);
    icon_weapon_transform.set_scale(Vector3::new(icon_scale, icon_scale, 0.0));

    // White shows the sprite as normal.
    // You can change the color at any point to modify the sprite's tint.
    let icon_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

    let weapon_icon = world
        .create_entity()
        .with(PlayerWeaponIcon::new(player_index, weapon_stats.weapon_type.clone()))
        .with(weapon_sprite)
        .with(icon_weapon_transform)
        .with(icon_tint)
        .with(Transparent)
        .build();


    //Create actual Player with Vehicle and Weapon
    world
        .create_entity()
        .with(vehicle_transform)
        .with(vehicle_sprite_render)
        .with(Vehicle::new(player_status_text, 
            health_entity,
            armor_entity,
            shield_entity,
            repair_entity,
        ))
        .with(Weapon::new(
            weapon_name,
            weapon_icon,
            weapon_stats,
        ))
        .with(Player::new(player_index, is_bot))
        .build();
    
}
