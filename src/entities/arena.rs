use amethyst::core::math::Vector3;
use amethyst::{
    assets::Handle,
    core::transform::Transform,
    ecs::prelude::World,
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};

use std::f32::consts::PI;

use crate::components::{Hitbox, HitboxShape};
use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};

pub fn initialise_arena_walls(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    //bottom UI wall
    let arena_ui_height = ARENA_HEIGHT + UI_HEIGHT;

    let mut wall_transform = Transform::default();
    wall_transform.set_translation_xyz(0.0, UI_HEIGHT - 1.0, 0.0);
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

    //UI divider walls

    let dx = 32.;
    let dx2 = 4.;

    for idx in 0..3 {
        let mut ui_div_wall_transform = Transform::default();
        ui_div_wall_transform.set_translation_xyz(
            100. + (idx as f32) * (3.0 * dx + dx2),
            UI_HEIGHT - 18.0,
            0.0,
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

    //central circle
    let mut circle_transform = Transform::default();
    let scale = 4.0;

    circle_transform.set_translation_xyz(ARENA_WIDTH / 2.0, arena_ui_height / 2.0, 0.0);
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
        ))
        .build();

    //outer circles
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
            .with(Hitbox::new(
                20.0 * scale,
                20.0 * scale,
                0.0,
                HitboxShape::Circle,
            ))
            .build();
    }
}
