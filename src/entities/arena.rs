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
use crate::resources::{GameModeSetup, GameModes, ArenaNavMesh, ArenaInvertedNavMesh};

pub fn initialize_arena_walls(
    world: &mut World,
    sprite_sheet_handle: Handle<SpriteSheet>,
    texture_sheet_handle: Handle<SpriteSheet>,
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


    //positions to place circular wall objects
    let mut wall_objects_x_y_scale: Vec<(f32, f32, f32)> = Vec::new();
    let mut nav_mesh_vertices: Vec<(f32, f32, f32)> = Vec::new();
    let mut nav_mesh_triangles: Vec<(usize, usize, usize)> = Vec::new();
    let mut nav_mesh_rect_vertices_x_y: Vec<(f32, f32, f32, f32, f32, f32, f32, f32)> = Vec::new();

    let debug_line_z = 0.5;
    let scale_mult = 10.0;
    let nav_mesh_offset = scale_mult;

    if game_mode == GameModes::Race {
        //the visual "start/finish line"
        let mut finish_line_transform = Transform::default();
        let scale = 4.0;

        finish_line_transform.set_translation_xyz(
            ARENA_WIDTH - scale_mult * scale,
            arena_ui_height / 2.0,
            -0.02,
        );
        finish_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let finish_line_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 30,
        };

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(finish_line_transform)
            .with(finish_line_sprite_render)
            .build();

        //the crossed "start/finish line" hitbox
        let mut finish_line_transform = Transform::default();
        let scale = 4.0;

        finish_line_transform.set_translation_xyz(
            ARENA_WIDTH - scale_mult * scale,
            arena_ui_height / 2.0 + 2.0 * scale,
            -0.02,
        );
        finish_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(finish_line_transform)
            .with(Hitbox::new(
                20.0 * scale,
                2.0 * scale,
                0.0,
                HitboxShape::Rectangle,
                false,
                false,
                RaceCheckpointType::Lap,
                0,
                false,
            ))
            .build();


        //1st "checkpoint line"
        let mut checkpoint_line_transform = Transform::default();
        let scale = 4.0;

        checkpoint_line_transform.set_translation_xyz(scale_mult * scale, arena_ui_height / 2.0, -0.02);
        checkpoint_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let checkpoint_line_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 31,
        };

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(checkpoint_line_transform)
            .with(checkpoint_line_sprite_render)
            .with(Hitbox::new(
                20.0 * scale,
                2.0 * scale,
                0.0,
                HitboxShape::Rectangle,
                false,
                false,
                RaceCheckpointType::Checkpoint,
                1,
                false,
            ))
            .build();


        //2nd "checkpoint line"
        let mut checkpoint_line_transform = Transform::default();
        let scale = 4.0;

        checkpoint_line_transform.set_rotation_2d(PI/2.0);
        checkpoint_line_transform.set_translation_xyz(ARENA_WIDTH/2.0, 20.0*scale -5.0, -0.02);
        checkpoint_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let checkpoint_line_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 31,
        };

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(checkpoint_line_transform)
            .with(checkpoint_line_sprite_render)
            .with(Hitbox::new(
                20.0 * scale,
                2.0 * scale,
                0.0,
                HitboxShape::Rectangle,
                false,
                false,
                RaceCheckpointType::Checkpoint,
                2,
                false,
            ))
            .build();

        

        //track layout
        let scale = 4.0;

        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0, arena_ui_height / 2.0, scale));
        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0 + 20.0 * scale, arena_ui_height / 2.0, scale));
        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0 + 20.0 * scale, arena_ui_height / 2.0 - 20.0 * scale, scale));
        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0 + 20.0 * scale, arena_ui_height / 2.0 + 20.0 * scale, scale));
        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0 + 40.0 * scale, arena_ui_height / 2.0 + 45.0 * scale, scale));
        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0 - 40.0 * scale, arena_ui_height / 2.0 + 45.0 * scale, scale));
        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0 - 20.0 * scale, arena_ui_height / 2.0, scale));
        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0 - 20.0 * scale, arena_ui_height / 2.0 + 20.0 * scale, scale));
        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0 - 30.0 * scale, arena_ui_height / 2.0 - 35.0 * scale, scale));
        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0, arena_ui_height / 2.0 - 20.0 * scale, scale));
        wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0, arena_ui_height / 2.0 + 45.0 * scale, scale));


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

            let offset = scale_mult*scale + nav_mesh_offset;

            nav_mesh_rect_vertices_x_y.push((
                ARENA_WIDTH / 2.0 - offset, arena_ui_height / 2.0 - offset,
                ARENA_WIDTH / 2.0 + offset, arena_ui_height / 2.0 - offset,
                ARENA_WIDTH / 2.0 + offset, arena_ui_height / 2.0 + offset,
                ARENA_WIDTH / 2.0 - offset, arena_ui_height / 2.0 + offset
            ));

        } else {
            //central arena wall circle
            let scale = 4.0;

            wall_objects_x_y_scale.push((ARENA_WIDTH / 2.0, arena_ui_height / 2.0, scale));
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

            wall_objects_x_y_scale.push((starting_x, starting_y, scale));

            let offset = scale_mult*scale + nav_mesh_offset;

            if idx == 0 {
                //left side
                nav_mesh_rect_vertices_x_y.push((
                    0.0, UI_HEIGHT,
                    0.0, ARENA_HEIGHT,
                    starting_x - offset, ARENA_HEIGHT, 
                    starting_x - offset, UI_HEIGHT, 
                )); 
                
                // //above
                // nav_mesh_rect_vertices_x_y.push((
                //     starting_x - offset, starting_y + scale*15.0,
                //     starting_x - offset, ARENA_HEIGHT,
                //     starting_x + offset, ARENA_HEIGHT,
                //     starting_x + offset, starting_y + scale*15.0,
                // ));

                // //below
                // nav_mesh_rect_vertices_x_y.push((
                //     starting_x - offset, starting_y - scale*15.0,
                //     starting_x - offset, UI_HEIGHT,
                //     starting_x + offset, UI_HEIGHT,
                //     starting_x + offset, starting_y - scale*15.0,
                // ));

                // //right side
                // nav_mesh_rect_vertices_x_y.push((
                //     starting_x + offset, UI_HEIGHT,
                //     starting_x + offset, ARENA_HEIGHT,
                //     ARENA_WIDTH / 2.0 - (scale_mult*4.0 + nav_mesh_offset), ARENA_HEIGHT, 
                //     ARENA_WIDTH / 2.0 - (scale_mult*4.0 + nav_mesh_offset), UI_HEIGHT, 
                // )); 
            }
            else if idx == 1 {
                // //left side
                // nav_mesh_rect_vertices_x_y.push((
                //     ARENA_WIDTH / 2.0 - (scale_mult*4.0 + nav_mesh_offset), ARENA_HEIGHT,
                //     starting_x - offset, ARENA_HEIGHT,
                //     starting_x - offset, arena_ui_height / 2.0 + (scale_mult*4.0 + nav_mesh_offset),
                //     ARENA_WIDTH / 2.0 - (scale_mult*4.0 + nav_mesh_offset), arena_ui_height / 2.0 + (scale_mult*4.0 + nav_mesh_offset),
                // )); 
            }
        }
    }

    for (x, y, scale) in wall_objects_x_y_scale {
        let mut circle_transform = Transform::default();
        
        circle_transform.set_translation_xyz(x, y, 0.38);
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


        //Build inverse navigation mesh    
        let fetched_arena_inv_nav_mesh = world.try_fetch_mut::<ArenaInvertedNavMesh>();

        if let Some(mut arena_inv_nav_mesh) = fetched_arena_inv_nav_mesh {
            arena_inv_nav_mesh.vertices.push((x, y, debug_line_z));
            arena_inv_nav_mesh.vertices.push((x - scale_mult*scale, y, debug_line_z));
            arena_inv_nav_mesh.vertices.push((x, y + scale_mult*scale, debug_line_z));

            let vertices_length = arena_inv_nav_mesh.vertices.clone().len();

            arena_inv_nav_mesh.triangles.push((vertices_length-3, vertices_length-2, vertices_length-1));
        }
    }


    //Divide navigation rectangles into navigation mesh triangles
    for (x1, y1, x2, y2, x3, y3, x4, y4) in nav_mesh_rect_vertices_x_y.iter() {
        nav_mesh_vertices.push((*x1, *y1, debug_line_z)); //len - 4
        nav_mesh_vertices.push((*x2, *y2, debug_line_z)); //len - 3
        nav_mesh_vertices.push((*x3, *y3, debug_line_z)); //len - 2
        nav_mesh_vertices.push((*x4, *y4, debug_line_z)); //len - 1

        let vertices_length = nav_mesh_vertices.clone().len();
    
        nav_mesh_triangles.push((vertices_length - 4, vertices_length - 1, vertices_length - 2));
        nav_mesh_triangles.push((vertices_length - 4, vertices_length - 3, vertices_length - 2));
    }

    //Store navigation mesh    
    let fetched_arena_nav_mesh = world.try_fetch_mut::<ArenaNavMesh>();

    if let Some(mut arena_nav_mesh) = fetched_arena_nav_mesh {
        arena_nav_mesh.vertices = nav_mesh_vertices.clone();
        arena_nav_mesh.triangles = nav_mesh_triangles.clone();
    }
}
