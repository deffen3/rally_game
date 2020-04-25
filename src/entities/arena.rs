use amethyst::{
    core::transform::Transform,
    assets::{Handle},
    renderer::{SpriteRender, SpriteSheet},
    ecs::prelude::{World},
    prelude::*,
};
use amethyst::core::math::Vector3;

use crate::rally::{ARENA_WIDTH, ARENA_HEIGHT, UI_HEIGHT};

use crate::components::{Hitbox, HitboxShape};


pub fn initialise_arena_walls(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>,) {
    let ARENA_UI_HEIGHT = ARENA_HEIGHT + UI_HEIGHT;

    let mut wall_transform = Transform::default();
    wall_transform.set_translation_xyz(0.0, UI_HEIGHT-1.0, 0.0);
    wall_transform.set_scale(Vector3::new(40.0, 1.0, 0.0));

    let wall_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 13,
    };

    world
        .create_entity()
        .with(Hitbox::new(20.0, 2.0, 0.0, HitboxShape::Rectangle))
        .with(wall_transform)
        .with(wall_sprite_render)
        .build();


    //central circle
    let mut circle_transform = Transform::default();
    let scale = 4.0;
    
    circle_transform.set_translation_xyz(ARENA_WIDTH/2.0, ARENA_UI_HEIGHT/2.0, 0.0);
    circle_transform.set_scale(Vector3::new(scale, scale, 0.0));

    let circle_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 14,
    };

    world
        .create_entity()
        .with(circle_transform)
        .with(circle_sprite_render)
        .with(Hitbox::new(20.0 * scale, 20.0 * scale, 0.0, HitboxShape::Circle))
        .build();


    //outer circles
    let spacing_factor = 5.0;
    let scale = 2.0;

    for idx in 0..4 {
        let (starting_x, starting_y) = match idx {
            0 => (ARENA_WIDTH / spacing_factor, ARENA_UI_HEIGHT / 2.0),
            1 => (ARENA_WIDTH / 2.0, ARENA_UI_HEIGHT / spacing_factor),
            2 => (ARENA_WIDTH - (ARENA_WIDTH / spacing_factor), ARENA_UI_HEIGHT / 2.0),
            3 => (ARENA_WIDTH / 2.0, ARENA_UI_HEIGHT - (ARENA_UI_HEIGHT / spacing_factor)),
            _ => (ARENA_WIDTH / spacing_factor, ARENA_UI_HEIGHT / spacing_factor),
        };

        let mut circle_transform = Transform::default();
        circle_transform.set_translation_xyz(starting_x, starting_y, 0.0);
        circle_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let circle_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 14,
        };

        world
            .create_entity()
            .with(circle_transform)
            .with(circle_sprite_render)
            .with(Hitbox::new(20.0 * scale, 20.0 * scale, 0.0, HitboxShape::Circle))
            .build();
    }
}