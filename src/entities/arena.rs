use amethyst::core::math::Vector3;
use amethyst::{
    assets::Handle,
    core::transform::Transform,
    ecs::prelude::World,
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};
use amethyst::renderer::{
    Transparent,
    palette::Srgba,
    resources::Tint,
};

use std::f32::consts::PI;

use crate::components::{Hitbox, HitboxShape};
use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT, GAME_MODE, GameModes};

pub fn initialize_arena_walls(
        world: &mut World, 
        sprite_sheet_handle: Handle<SpriteSheet>,
        texture_sheet_handle: Handle<SpriteSheet>
    ) {
    
    let arena_ui_height = ARENA_HEIGHT + UI_HEIGHT;

    //arena floor
    let mut floor_transform = Transform::default();
    floor_transform.set_translation_xyz(ARENA_WIDTH/2.0, arena_ui_height/2.0, -0.05);
    floor_transform.set_scale(Vector3::new(6.25, 5.75, 0.0));

    let floor_texture_render = SpriteRender {
        sprite_sheet: texture_sheet_handle,
        sprite_number: 0,
    };

    world
        .create_entity()
        .with(floor_transform)
        .with(floor_texture_render)
        .build();

    //bottom UI wall
    let mut wall_transform = Transform::default();
    wall_transform.set_translation_xyz(0.0, UI_HEIGHT - 1.0, 0.39);
    wall_transform.set_scale(Vector3::new(40.0, 1.0, 0.0));

    let wall_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 13,
    };

    world
        .create_entity()
        .with(Hitbox::new(20.0, 2.0, 0.0, HitboxShape::Rectangle, true, false))
        .with(wall_transform)
        .with(wall_sprite_render)
        .build();


    //bottom UI background
    let mut ui_back_transform = Transform::default();
    ui_back_transform.set_translation_xyz(0.0, UI_HEIGHT/2.0 - 1.0, 0.35);
    ui_back_transform.set_scale(Vector3::new(40.0, 9.0, 0.0));

    let ui_back_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 24,
    };

    world
        .create_entity()
        .with(ui_back_transform)
        .with(ui_back_sprite_render)
        .build();
    


    //UI divider walls

    let dx = 32.;
    let dx2 = 4.;

    for idx in 0..3 {
        let mut ui_div_wall_transform = Transform::default();
        ui_div_wall_transform.set_translation_xyz(
            100. + (idx as f32) * (3.0 * dx + dx2),
            UI_HEIGHT - 18.0,
            0.4,
        );
        ui_div_wall_transform.set_scale(Vector3::new(1.7, 1.0, 0.0));
        ui_div_wall_transform.set_rotation_2d(PI / 2.0);

        let wall_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 13,
        };

        world
            .create_entity()
            .with(ui_div_wall_transform)
            .with(wall_sprite_render)
            .build();
    }


    if GAME_MODE == GameModes::KingOfTheHill {
        //the "hill"
        let mut circle_transform = Transform::default();
        let scale = 4.0;

        circle_transform.set_translation_xyz(ARENA_WIDTH / 2.0, arena_ui_height / 2.0, -0.02);
        circle_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let circle_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 29,
        };


        // White shows the sprite as normal.
        // You can change the color at any point to modify the sprite's tint.
        let king_tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

        world
            .create_entity()
            .with(circle_transform)
            .with(circle_sprite_render)
            .with(Hitbox::new(
                20.0 * scale,
                20.0 * scale,
                0.0,
                HitboxShape::Circle,
                false,
                true,
            ))
            .with(Transparent)
            .with(king_tint)
            .build();

    } else {
        //central arena wall circle
        let mut circle_transform = Transform::default();
        let scale = 4.0;

        circle_transform.set_translation_xyz(ARENA_WIDTH / 2.0, arena_ui_height / 2.0, 0.38);
        circle_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let circle_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 14,
        };

        world
            .create_entity()
            .with(circle_transform)
            .with(circle_sprite_render)
            .with(Hitbox::new(
                20.0 * scale,
                20.0 * scale,
                0.0,
                HitboxShape::Circle,
                true,
                false,
            ))
            .build();
    }

    //outer arena wall circles
    let spacing_factor = 5.0;
    let scale = 2.0;

    for idx in 0..4 {
        let (starting_x, starting_y) = match idx {
            0 => (ARENA_WIDTH / spacing_factor, arena_ui_height / 2.0),
            1 => (ARENA_WIDTH / 2.0, arena_ui_height / spacing_factor),
            2 => (
                ARENA_WIDTH - (ARENA_WIDTH / spacing_factor),
                arena_ui_height / 2.0,
            ),
            3 => (
                ARENA_WIDTH / 2.0,
                arena_ui_height - (arena_ui_height / spacing_factor),
            ),
            _ => (
                ARENA_WIDTH / spacing_factor,
                arena_ui_height / spacing_factor,
            ),
        };

        let mut circle_transform = Transform::default();
        circle_transform.set_translation_xyz(starting_x, starting_y, 0.8);
        circle_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let circle_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 14,
        };

        world
            .create_entity()
            .with(circle_transform)
            .with(circle_sprite_render)
            .with(Hitbox::new(
                20.0 * scale,
                20.0 * scale,
                0.0,
                HitboxShape::Circle,
                true,
                false,
            ))
            .build();
    }
}
