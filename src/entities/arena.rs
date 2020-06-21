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

use std::f32::{consts::PI};

use crate::components::{Hitbox, HitboxShape, RaceCheckpointType};
use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};
use crate::resources::{
    GameModeSetup, GameModes, 
    ArenaNavMesh, ArenaInvertedNavMesh, ArenaNavMeshFinal
};

use navmesh::{NavMesh, NavVec3, NavTriangle};


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
    let mut arena_circle_objects_x_y_scale: Vec<(f32, f32, f32)> = Vec::new();

    let debug_line_z = 0.0;
    let scale_mult = 10.0;
    let nav_mesh_offset = 7.0;

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

        checkpoint_line_transform.set_rotation_2d(-PI/2.0);
        checkpoint_line_transform.set_translation_xyz(ARENA_WIDTH/2.0, ARENA_HEIGHT - 20.0*scale, -0.02);
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
                2,
                false,
            ))
            .build();


        //3rd "checkpoint line"
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
                3,
                false,
            ))
            .build();

        

        //track layout
        let scale = 4.0;

        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0, arena_ui_height / 2.0 + 8.0 * scale, scale));
        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0 + 20.0 * scale, arena_ui_height / 2.0, scale));
        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0 + 20.0 * scale, arena_ui_height / 2.0 - 20.0 * scale, scale));
        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0 + 20.0 * scale, arena_ui_height / 2.0 + 20.0 * scale, scale));
        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0 + 40.0 * scale, arena_ui_height / 2.0 + 45.0 * scale, scale));
        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0 - 40.0 * scale, arena_ui_height / 2.0 + 45.0 * scale, scale));
        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0 - 20.0 * scale, arena_ui_height / 2.0, scale));
        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0 - 20.0 * scale, arena_ui_height / 2.0 + 20.0 * scale, scale));
        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0 - 30.0 * scale, arena_ui_height / 2.0 - 35.0 * scale, scale));
        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0, arena_ui_height / 2.0 - 20.0 * scale, scale));
        arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0, arena_ui_height / 2.0 + 45.0 * scale, scale));


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
            let scale = 4.0;

            arena_circle_objects_x_y_scale.push((ARENA_WIDTH / 2.0, arena_ui_height / 2.0, scale));
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

            arena_circle_objects_x_y_scale.push((starting_x, starting_y, scale));
        }
    }


    let mut nav_mesh_grid_xs: Vec<f32> = Vec::new();
    nav_mesh_grid_xs.push(0.0 + nav_mesh_offset);
    nav_mesh_grid_xs.push(ARENA_WIDTH - nav_mesh_offset);

    let mut nav_mesh_grid_ys: Vec<f32> = Vec::new();
    nav_mesh_grid_ys.push(UI_HEIGHT + nav_mesh_offset);
    nav_mesh_grid_ys.push(ARENA_HEIGHT - nav_mesh_offset);

    let mut nav_mesh_grid_drop: Vec<(f32, f32, f32, f32)> = Vec::new();


    for (x, y, scale) in arena_circle_objects_x_y_scale {
        //add visual sprite
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

        
        
        //setup nav mesh grid
        let offset = scale_mult*scale + nav_mesh_offset;

        let xr_minus = (x - offset).round();
        let xr_plus = (x + offset).round();

        let xr_minus_find_index = nav_mesh_grid_xs.iter().position(|&r| r == xr_minus);
        let xr_plus_find_index = nav_mesh_grid_xs.iter().position(|&r| r == xr_plus);

        if xr_minus_find_index.is_none() {
            nav_mesh_grid_xs.push(xr_minus);
        }
        if xr_plus_find_index.is_none() {
            nav_mesh_grid_xs.push(xr_plus);
        }


        let yr_minus = (y - offset).round();
        let yr_plus = (y + offset).round();

        let yr_minus_find_index = nav_mesh_grid_ys.iter().position(|&r| r == yr_minus);
        let yr_plus_find_index = nav_mesh_grid_ys.iter().position(|&r| r == yr_plus);

        if yr_minus_find_index.is_none() {
            nav_mesh_grid_ys.push(yr_minus);
        }
        if yr_plus_find_index.is_none() {
            nav_mesh_grid_ys.push(yr_plus);
        }


        nav_mesh_grid_drop.push((xr_minus, xr_plus, yr_minus, yr_plus));



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



    //Build navigation mesh from grid
    let mut nav_mesh_vertices: Vec<(f32, f32, f32)> = Vec::new();
    let mut nav_mesh_triangles: Vec<(usize, usize, usize)> = Vec::new();


    nav_mesh_grid_xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let xs_len = nav_mesh_grid_xs.len();

    nav_mesh_grid_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let ys_len = nav_mesh_grid_xs.len();

    log::info!("{} {}", xs_len, ys_len);
    log::info!("{:?} {:?}", nav_mesh_grid_xs, nav_mesh_grid_ys);
    log::info!("{:?}", nav_mesh_grid_drop);

    let mut grid_drops: usize = 0;

    for (y_idx, y) in nav_mesh_grid_ys.iter().enumerate() {
        for (x_idx, x) in nav_mesh_grid_xs.iter().enumerate() {
            nav_mesh_vertices.push((*x, *y, debug_line_z));

            let vertex_idx = nav_mesh_vertices.len() - 1;

            if (x_idx > 0) && (y_idx > 0) {
                let xr_minus = nav_mesh_vertices[vertex_idx - 1].0; //bottom-left x
                let xr_plus = nav_mesh_vertices[vertex_idx - xs_len].0; //top-right x
                let yr_minus = nav_mesh_vertices[vertex_idx - 1].1; //bottom-left y
                let yr_plus = nav_mesh_vertices[vertex_idx - xs_len].1; //top-right y

                let mut dropped = false;
                for (drop_x_minus, drop_x_plus, drop_y_minus, drop_y_plus) in nav_mesh_grid_drop.iter() {
                    if xr_minus >= *drop_x_minus && xr_minus <= *drop_x_plus && 
                            xr_plus >= *drop_x_minus && xr_plus <= *drop_x_plus &&
                            yr_minus >= *drop_y_minus && yr_minus <= *drop_y_plus &&
                            yr_plus >= *drop_y_minus && yr_plus <= *drop_y_plus {
                        dropped = true;
                        grid_drops += 1;
                        break;
                    }
                }

                if !dropped {
                    nav_mesh_triangles.push((vertex_idx - 1, vertex_idx - xs_len - 1, vertex_idx - xs_len));
                    nav_mesh_triangles.push((vertex_idx - 1, vertex_idx, vertex_idx - xs_len));
                }
            }
        }
    }

    log::info!("{} == {}", nav_mesh_vertices.len(), xs_len * ys_len);
    log::info!("{} == {}", nav_mesh_triangles.len(), 2 * (xs_len-1) * (ys_len-1) - 2*grid_drops);

    //assert!(nav_mesh_vertices.len() == xs_len * ys_len);
    //assert!(nav_mesh_triangles.len() == 2 * (xs_len-1) * (ys_len-1) - 2*grid_drops);


    //Store navigation mesh
    let fetched_arena_nav_mesh = world.try_fetch_mut::<ArenaNavMesh>();

    if let Some(mut arena_nav_mesh) = fetched_arena_nav_mesh {
        arena_nav_mesh.vertices = nav_mesh_vertices.clone();
        arena_nav_mesh.triangles = nav_mesh_triangles.clone();

        let fetched_arena_nav_mesh_final = world.try_fetch_mut::<ArenaNavMeshFinal>();

        if let Some(mut arena_nav_mesh_final) = fetched_arena_nav_mesh_final {

            let mut nav_vecs: Vec<NavVec3> = Vec::new();
            let mut nav_triangles: Vec<NavTriangle> = Vec::new();

            for (x,y,z) in arena_nav_mesh.vertices.iter() {
                nav_vecs.push(NavVec3::new(*x, *y, *z));
            }

            for (v1, v2, v3) in arena_nav_mesh.triangles.iter() {
                nav_triangles.push(NavTriangle {
                    first: *v1 as u32,
                    second: *v2 as u32,
                    third: *v3 as u32
                });
            }

            arena_nav_mesh_final.mesh = Some(NavMesh::new(
                nav_vecs.clone(),
                nav_triangles.clone()
            ).unwrap());
        }
    }
}
