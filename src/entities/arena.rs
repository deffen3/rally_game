use amethyst::core::math::Vector3;
use amethyst::renderer::{palette::Srgba, resources::Tint, Transparent};
use amethyst::{
    assets::Handle,
    core::transform::Transform,
    ecs::prelude::World,
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
    utils::removal::Removal,
};

use std::f32::consts::PI;

use crate::components::{Hitbox, HitboxShape, RaceCheckpointType};
use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};
use crate::resources::{GameModeSetup, GameModes};

pub fn initialize_arena_walls(
    world: &mut World,
    sprite_sheet_handle: Handle<SpriteSheet>,
    texture_sheet_handle: Handle<SpriteSheet>,
    //game_mode: GameModes,
) {
    let game_mode;
    {
        let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();

        if let Some(game_mode_setup) = fetched_game_mode_setup {
            game_mode = game_mode_setup.game_mode.clone();
        } else {
            game_mode = GameModes::ClassicGunGame;
        }
    }

    let arena_ui_height = ARENA_HEIGHT + UI_HEIGHT;

    //arena floor
    let mut floor_transform = Transform::default();
    floor_transform.set_translation_xyz(ARENA_WIDTH / 2.0, arena_ui_height / 2.0, -0.05);
    floor_transform.set_scale(Vector3::new(6.25, 5.75, 0.0));

    let floor_texture_render = SpriteRender {
        sprite_sheet: texture_sheet_handle,
        sprite_number: 0,
    };

    world
        .create_entity()
        .with(Removal::new(0 as u32))
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
        .with(Removal::new(0 as u32))
        .with(Hitbox::new(
            20.0,
            2.0,
            0.0,
            HitboxShape::Rectangle,
            true,
            false,
            RaceCheckpointType::NotCheckpoint,
            0,
            false,
        ))
        .with(wall_transform)
        .with(wall_sprite_render)
        .build();

    //bottom UI background
    let mut ui_back_transform = Transform::default();
    ui_back_transform.set_translation_xyz(0.0, UI_HEIGHT / 2.0 - 1.0, 0.35);
    ui_back_transform.set_scale(Vector3::new(40.0, 9.0, 0.0));

    let ui_back_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 24,
    };

    world
        .create_entity()
        .with(Removal::new(0 as u32))
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
            .with(Removal::new(0 as u32))
            .with(ui_div_wall_transform)
            .with(wall_sprite_render)
            .build();
    }

    if game_mode == GameModes::Race {
        //the "start/finish line"
        let mut finsh_line_transform = Transform::default();
        let scale = 4.0;

        finsh_line_transform.set_translation_xyz(
            ARENA_WIDTH - 10.0 * scale,
            arena_ui_height / 2.0,
            -0.02,
        );
        finsh_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let finish_line_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 30,
        };

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(finsh_line_transform)
            .with(finish_line_sprite_render)
            .with(Hitbox::new(
                20.0 * scale,
                2.0 * scale,
                0.0,
                HitboxShape::Rectangle,
                false,
                false,
                RaceCheckpointType::LapStart,
                0,
                false,
            ))
            .build();

        //the crossed "start/finish line" hitbox
        let mut finsh_line_transform = Transform::default();
        let scale = 4.0;

        finsh_line_transform.set_translation_xyz(
            ARENA_WIDTH - 10.0 * scale,
            arena_ui_height / 2.0 + 2.0 * scale,
            -0.02,
        );
        finsh_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(finsh_line_transform)
            .with(Hitbox::new(
                20.0 * scale,
                2.0 * scale,
                0.0,
                HitboxShape::Rectangle,
                false,
                false,
                RaceCheckpointType::LapFinish,
                0,
                false,
            ))
            .build();

        //hidden checkpoint
        let mut finsh_line_transform = Transform::default();
        let scale = 4.0;

        finsh_line_transform.set_translation_xyz(
            ARENA_WIDTH - 10.0 * scale,
            arena_ui_height / 2.0 + 6.0 * scale,
            -0.02,
        );
        finsh_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(finsh_line_transform)
            .with(Hitbox::new(
                20.0 * scale,
                2.0 * scale,
                0.0,
                HitboxShape::Rectangle,
                false,
                false,
                RaceCheckpointType::CheckpointStart,
                1,
                false,
            ))
            .build();

        //hidden checkpoint crossed
        let mut finsh_line_transform = Transform::default();
        let scale = 4.0;

        finsh_line_transform.set_translation_xyz(
            ARENA_WIDTH - 10.0 * scale,
            arena_ui_height / 2.0 + 8.0 * scale,
            -0.02,
        );
        finsh_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(finsh_line_transform)
            .with(Hitbox::new(
                20.0 * scale,
                2.0 * scale,
                0.0,
                HitboxShape::Rectangle,
                false,
                false,
                RaceCheckpointType::CheckpointFinish,
                1,
                false,
            ))
            .build();

        //a visual "checkpoint line"
        let mut finsh_line_transform = Transform::default();
        let scale = 4.0;

        finsh_line_transform.set_translation_xyz(10.0 * scale, arena_ui_height / 2.0, -0.02);
        finsh_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let finish_line_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 31,
        };

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(finsh_line_transform)
            .with(finish_line_sprite_render)
            .with(Hitbox::new(
                20.0 * scale,
                2.0 * scale,
                0.0,
                HitboxShape::Rectangle,
                false,
                false,
                RaceCheckpointType::CheckpointStart,
                2,
                false,
            ))
            .build();

        //visual crossed "checkpoint line" hitbox
        let mut finsh_line_transform = Transform::default();
        let scale = 4.0;

        finsh_line_transform.set_translation_xyz(
            10.0 * scale,
            arena_ui_height / 2.0 - 2.0 * scale,
            -0.02,
        );
        finsh_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(finsh_line_transform)
            .with(Hitbox::new(
                20.0 * scale,
                2.0 * scale,
                0.0,
                HitboxShape::Rectangle,
                false,
                false,
                RaceCheckpointType::CheckpointFinish,
                2,
                false,
            ))
            .build();

        //track layout
        let mut track_transform = Transform::default();
        let scale = 4.0;

        track_transform.set_translation_xyz(
            ARENA_WIDTH - 20.0 * scale - 5.0 / 2.0 * scale,
            arena_ui_height / 2.0,
            -0.02,
        );
        track_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let track_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 33,
        };

        let shape = HitboxShape::Rectangle;

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(track_transform)
            .with(track_sprite_render)
            .with(Hitbox::new(
                5.0 * scale,
                10.0 * scale,
                0.0,
                shape,
                true,
                false,
                RaceCheckpointType::NotCheckpoint,
                0,
                false,
            ))
            .build();

        let mut track_transform = Transform::default();
        let scale = 4.0;

        track_transform.set_translation_xyz(
            ARENA_WIDTH - 20.0 * scale - 5.0 / 2.0 * scale,
            arena_ui_height / 2.0 + 10.0 * scale,
            -0.02,
        );
        track_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let track_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 33,
        };

        let shape = HitboxShape::Rectangle;

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(track_transform)
            .with(track_sprite_render)
            .with(Hitbox::new(
                5.0 * scale,
                10.0 * scale,
                0.0,
                shape,
                true,
                false,
                RaceCheckpointType::NotCheckpoint,
                0,
                false,
            ))
            .build();

        let mut track_transform = Transform::default();
        let scale = 4.0;

        track_transform.set_translation_xyz(
            ARENA_WIDTH - 20.0 * scale - 5.0 / 2.0 * scale,
            arena_ui_height / 2.0 - 10.0 * scale,
            -0.02,
        );
        track_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let track_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 33,
        };

        let shape = HitboxShape::Rectangle;

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(track_transform)
            .with(track_sprite_render)
            .with(Hitbox::new(
                5.0 * scale,
                10.0 * scale,
                0.0,
                shape,
                true,
                false,
                RaceCheckpointType::NotCheckpoint,
                0,
                false,
            ))
            .build();

        let mut track_transform = Transform::default();
        let scale = 4.0;

        track_transform.set_translation_xyz(
            20.0 * scale + 5.0 / 2.0 * scale,
            arena_ui_height / 2.0,
            -0.02,
        );
        track_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let track_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 33,
        };

        let shape = HitboxShape::Rectangle;

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(track_transform)
            .with(track_sprite_render)
            .with(Hitbox::new(
                5.0 * scale,
                10.0 * scale,
                0.0,
                shape,
                true,
                false,
                RaceCheckpointType::NotCheckpoint,
                0,
                false,
            ))
            .build();
    } else {
        if game_mode == GameModes::KingOfTheHill {
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
                .with(Removal::new(0 as u32))
                .with(circle_transform)
                .with(circle_sprite_render)
                .with(Hitbox::new(
                    20.0 * scale,
                    20.0 * scale,
                    0.0,
                    HitboxShape::Circle,
                    false,
                    true,
                    RaceCheckpointType::NotCheckpoint,
                    0,
                    false,
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
                .with(Removal::new(0 as u32))
                .with(circle_transform)
                .with(circle_sprite_render)
                .with(Hitbox::new(
                    20.0 * scale,
                    20.0 * scale,
                    0.0,
                    HitboxShape::Circle,
                    true,
                    false,
                    RaceCheckpointType::NotCheckpoint,
                    0,
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
                .with(Removal::new(0 as u32))
                .with(circle_transform)
                .with(circle_sprite_render)
                .with(Hitbox::new(
                    20.0 * scale,
                    20.0 * scale,
                    0.0,
                    HitboxShape::Circle,
                    true,
                    false,
                    RaceCheckpointType::NotCheckpoint,
                    0,
                    false,
                ))
                .build();
        }
    }
}
