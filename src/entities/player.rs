use amethyst::{
    core::transform::Transform,
    assets::{Handle},
    renderer::{SpriteRender, SpriteSheet},
    ecs::prelude::{World},
    prelude::*,
};


use std::f32::consts::PI;

use crate::components::{
    Player, Vehicle, Weapon, WeaponTypes, build_standard_weapon,
};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};

pub fn intialize_player(
    world: &mut World, 
    sprite_sheet_handle: Handle<SpriteSheet>,
    player_index: usize,
    weapon_type: WeaponTypes,
) {
    let mut vehicle_transform = Transform::default();

    let spacing_factor = 5.0;

    let height = ARENA_HEIGHT + UI_HEIGHT;

    let (starting_rotation, starting_x, starting_y) = match player_index {
        0 => (-PI/4.0, ARENA_WIDTH / spacing_factor, height / spacing_factor),
        1 => (PI/2.0 + PI/4.0, ARENA_WIDTH - (ARENA_WIDTH / spacing_factor), height - (height / spacing_factor)),
        2 => (PI + PI/4.0, ARENA_WIDTH / spacing_factor, height - (height / spacing_factor)),
        3 => (PI/2.0 - PI/4.0, ARENA_WIDTH - (ARENA_WIDTH / spacing_factor), height / spacing_factor),
        _ => (-PI/4.0, ARENA_WIDTH / spacing_factor, height / spacing_factor),
    };

    vehicle_transform.set_rotation_2d(starting_rotation as f32);
    vehicle_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.0);

    let vehicle_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: player_index,
    };

    let (weapon_type,
        heat_seeking,
        heat_seeking_agility,
        attached,
        deployed,
        weapon_cooldown, 
        burst_shot_limit,
        burst_cooldown,
        weapon_shot_speed,
        damage,
        shield_damage_pct,
        armor_damage_pct,
        piercing_damage_pct,
        health_damage_pct,) = build_standard_weapon(weapon_type);

    world
        .create_entity()
        .with(vehicle_transform)
        .with(vehicle_sprite_render.clone())
        .with(Vehicle::new())
        .with(Weapon::new(weapon_type,
            heat_seeking,
            heat_seeking_agility,
            attached,
            deployed,
            weapon_cooldown, 
            burst_shot_limit,
            burst_cooldown,
            weapon_shot_speed,
            damage,
            shield_damage_pct,
            armor_damage_pct,
            piercing_damage_pct,
            health_damage_pct))
        .with(Player::new(player_index))
        .build();


    //I can build all of this as one entity, but then I get only one sprite.
    //if I separate it into three entities, then now my systems are broken as their
    //  is no relationship between these entities. Do I need to apply parent child relationships?
    //  Isn't this going against the purpose/elegance of ECS?



    //UI icon
    let mut icon_transform = Transform::default();

    let x = 15.;
    let y = UI_HEIGHT - 10.;
    let dx = 32.;
    let dx2 = 4.;

    let (starting_x) = match player_index {
        0 => (x),
        1 => (x + 3.0*dx + dx2),
        2 => (x + 6.0*dx + 2.0*dx2),
        3 => (x + 9.0*dx + 3.0*dx2),
        _ => (0.0),
    };

    icon_transform.set_rotation_2d(-PI/2.0);
    icon_transform.set_translation_xyz(starting_x as f32, y, 0.0);

    world
        .create_entity()
        .with(icon_transform)
        .with(vehicle_sprite_render)
        .build();
}