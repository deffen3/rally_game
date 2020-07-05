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

use crate::components::{
    ArenaElement, ArenaNames, ArenaStoreResource, ArenaProperties,
    Hitbox, HitboxShape, RaceCheckpointType, reform_weapon_spawner,
    ObstacleType,
};

use crate::resources::{
    GameModeSetup, 
    ArenaNavMesh, ArenaNavMeshFinal
};

use navmesh::{NavMesh, NavVec3, NavTriangle};






pub fn intialize_arena(
    world: &mut World,
    sprite_sheet_handle: Handle<SpriteSheet>,
    texture_sheet_handle: Handle<SpriteSheet>,
) {
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



    //Initialize Nav Mesh Grid
    let debug_line_z = 0.0;
    let nav_mesh_offset = 7.0;

    let mut nav_mesh_grid_xs: Vec<f32> = Vec::new();
    nav_mesh_grid_xs.push(0.0 + nav_mesh_offset);
    nav_mesh_grid_xs.push(0.0 + 3.0*nav_mesh_offset);
    nav_mesh_grid_xs.push(arena_properties.width - nav_mesh_offset);
    nav_mesh_grid_xs.push(arena_properties.width - 3.0*nav_mesh_offset);

    let mut nav_mesh_grid_ys: Vec<f32> = Vec::new();
    nav_mesh_grid_ys.push(0.0 + nav_mesh_offset);
    nav_mesh_grid_ys.push(0.0 + 3.0*nav_mesh_offset);
    nav_mesh_grid_ys.push(arena_properties.height - nav_mesh_offset);
    nav_mesh_grid_ys.push(arena_properties.height - 3.0*nav_mesh_offset);

    let mut nav_mesh_grid_drop: Vec<(f32, f32, f32, f32)> = Vec::new();




    //Build Arena from properties

    //Arena Floor
    for arena_floor in arena_properties.floor.iter() {
        let sprite_scale_mult = 64.0;
        let x_scale = arena_floor.width / sprite_scale_mult;
        let y_scale = arena_floor.height / sprite_scale_mult;

        let mut floor_transform = Transform::default();
        floor_transform.set_translation_xyz(arena_floor.x, arena_floor.y, -0.05);
        floor_transform.set_scale(Vector3::new(x_scale, y_scale, 0.0));

        let floor_texture_render = SpriteRender {
            sprite_sheet: texture_sheet_handle.clone(),
            sprite_number: 0,
        };

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(floor_transform)
            .with(floor_texture_render)
            .build();
    }



    //Arena Rectangles
    for arena_rect in arena_properties.arena_rectangles.iter() {
        let sprite_scale_mult = 20.0;
        let x_scale = arena_rect.width/2.0 / sprite_scale_mult;
        let y_scale = arena_rect.height/2.0 / sprite_scale_mult;

        //add visual sprite
        let mut transform = Transform::default();
        
        transform.set_rotation_2d(arena_rect.rotation/180.0 * PI);
        if arena_rect.obstacle_type == ObstacleType::Wall {
            transform.set_translation_xyz(arena_rect.x, arena_rect.y, 0.38);
        }
        else {
            transform.set_translation_xyz(arena_rect.x, arena_rect.y, -0.01);
        }
        transform.set_scale(Vector3::new(x_scale, y_scale, 0.0));
        


        if arena_rect.obstacle_type == ObstacleType::Wall {
            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: 71,
            };

            world
                .create_entity()
                .with(Removal::new(0 as u32))
                .with(transform)
                .with(sprite_render)
                .with(ArenaElement {
                    obstacle_type: arena_rect.obstacle_type,
                    is_hill: false,
                    checkpoint: RaceCheckpointType::NotCheckpoint,
                    checkpoint_id: 0,
                    is_weapon_box: false,
                    is_spawn_point: false,
                    is_weapon_spawn_point: false,
                    x: arena_rect.x,
                    y: arena_rect.y,
                    z: 0.0,
                    is_sprite: true,
                    sprite: 71,
                    sprite_scale: x_scale,
                    weapon_names: None,
                    first_spawn_time: None,
                    spawn_time: None,
                    spawn_timer: None,
                    ammo: None,
                    hitbox: Hitbox::new(
                        2.0*sprite_scale_mult * x_scale,
                        2.0*sprite_scale_mult * y_scale,
                        0.0,
                        HitboxShape::Rectangle,
                    ),
                    effects: None,
                })
                .build();
        }
        else {
            if let Some(arena_rect_effects) = arena_rect.effects {
                if arena_rect.obstacle_type == ObstacleType::Zone && arena_rect_effects.damage_rate > 0.0 {
                    let sprite_render = SpriteRender {
                        sprite_sheet: sprite_sheet_handle.clone(),
                        sprite_number: 73,
                    };

                    world
                        .create_entity()
                        .with(Removal::new(0 as u32))
                        .with(transform)
                        .with(sprite_render)
                        .with(ArenaElement {
                            obstacle_type: arena_rect.obstacle_type,
                            is_hill: false,
                            checkpoint: RaceCheckpointType::NotCheckpoint,
                            checkpoint_id: 0,
                            is_weapon_box: false,
                            is_spawn_point: false,
                            is_weapon_spawn_point: false,
                            x: arena_rect.x,
                            y: arena_rect.y,
                            z: 0.0,
                            is_sprite: true,
                            sprite: 71,
                            sprite_scale: x_scale,
                            weapon_names: None,
                            first_spawn_time: None,
                            spawn_time: None,
                            spawn_timer: None,
                            ammo: None,
                            hitbox: Hitbox::new(
                                2.0*sprite_scale_mult * x_scale,
                                2.0*sprite_scale_mult * y_scale,
                                0.0,
                                HitboxShape::Rectangle,
                            ),
                            effects: Some(arena_rect_effects),
                        })
                        .build();
                }
                else if arena_rect.obstacle_type == ObstacleType::Zone && arena_rect_effects.damage_rate < 0.0 { //healing
                    let sprite_render = SpriteRender {
                        sprite_sheet: sprite_sheet_handle.clone(),
                        sprite_number: 74,
                    };

                    world
                        .create_entity()
                        .with(Removal::new(0 as u32))
                        .with(transform)
                        .with(sprite_render)
                        .with(ArenaElement {
                            obstacle_type: arena_rect.obstacle_type,
                            is_hill: false,
                            checkpoint: RaceCheckpointType::NotCheckpoint,
                            checkpoint_id: 0,
                            is_weapon_box: false,
                            is_spawn_point: false,
                            is_weapon_spawn_point: false,
                            x: arena_rect.x,
                            y: arena_rect.y,
                            z: 0.0,
                            is_sprite: true,
                            sprite: 71,
                            sprite_scale: x_scale,
                            weapon_names: None,
                            first_spawn_time: None,
                            spawn_time: None,
                            spawn_timer: None,
                            ammo: None,
                            hitbox: Hitbox::new(
                                2.0*sprite_scale_mult * x_scale,
                                2.0*sprite_scale_mult * y_scale,
                                0.0,
                                HitboxShape::Rectangle,
                            ),
                            effects: Some(arena_rect_effects),
                        })
                        .build();
                }
                else if arena_rect.obstacle_type == ObstacleType::Zone && arena_rect_effects.accel_rate != 0.0 {
                    let sprite_render = SpriteRender {
                        sprite_sheet: sprite_sheet_handle.clone(),
                        sprite_number: 72,
                    };

                    world
                        .create_entity()
                        .with(Removal::new(0 as u32))
                        .with(transform)
                        .with(sprite_render)
                        .with(ArenaElement {
                            obstacle_type: arena_rect.obstacle_type,
                            is_hill: false,
                            checkpoint: RaceCheckpointType::NotCheckpoint,
                            checkpoint_id: 0,
                            is_weapon_box: false,
                            is_spawn_point: false,
                            is_weapon_spawn_point: false,
                            x: arena_rect.x,
                            y: arena_rect.y,
                            z: 0.0,
                            is_sprite: true,
                            sprite: 71,
                            sprite_scale: x_scale,
                            weapon_names: None,
                            first_spawn_time: None,
                            spawn_time: None,
                            spawn_timer: None,
                            ammo: None,
                            hitbox: Hitbox::new(
                                2.0*sprite_scale_mult * x_scale,
                                2.0*sprite_scale_mult * y_scale,
                                0.0,
                                HitboxShape::Rectangle,
                            ),
                            effects: Some(arena_rect_effects),
                        })
                        .build();
                }
            }
        }
            
        

        if arena_rect.obstacle_type == ObstacleType::Wall {
            //setup nav mesh grid
            let x_offset = sprite_scale_mult*x_scale + nav_mesh_offset;
        
            let xr_minus = (arena_rect.x - x_offset).round();
            let xr_plus = (arena_rect.x + x_offset).round();

            let xr_minus_find_index = nav_mesh_grid_xs.iter().position(|&r| r == xr_minus);
            let xr_plus_find_index = nav_mesh_grid_xs.iter().position(|&r| r == xr_plus);

            if xr_minus_find_index.is_none() {
                nav_mesh_grid_xs.push(xr_minus);
            }
            if xr_plus_find_index.is_none() {
                nav_mesh_grid_xs.push(xr_plus);
            }


            let y_offset = sprite_scale_mult*y_scale + nav_mesh_offset;

            let yr_minus = (arena_rect.y - y_offset).round();
            let yr_plus = (arena_rect.y + y_offset).round();

            let yr_minus_find_index = nav_mesh_grid_ys.iter().position(|&r| r == yr_minus);
            let yr_plus_find_index = nav_mesh_grid_ys.iter().position(|&r| r == yr_plus);

            if yr_minus_find_index.is_none() {
                nav_mesh_grid_ys.push(yr_minus);
            }
            if yr_plus_find_index.is_none() {
                nav_mesh_grid_ys.push(yr_plus);
            }


            nav_mesh_grid_drop.push((xr_minus, xr_plus, yr_minus, yr_plus));
        }
    }


    //Arena Circles
    for arena_circle in arena_properties.arena_circles.iter() {
        let sprite_scale_mult = 20.0;
        let scale = arena_circle.radius / sprite_scale_mult;

        //add visual sprite
        let mut circle_transform = Transform::default();
        
        circle_transform.set_translation_xyz(arena_circle.x, arena_circle.y, 0.38);
        circle_transform.set_scale(Vector3::new(scale, scale, 0.0));

        let circle_sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 70,
        };

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(circle_transform)
            .with(circle_sprite_render)
            .with(ArenaElement {
                obstacle_type: arena_circle.obstacle_type,
                is_hill: false,
                checkpoint: RaceCheckpointType::NotCheckpoint,
                checkpoint_id: 0,
                is_weapon_box: false,
                is_spawn_point: false,
                is_weapon_spawn_point: false,
                x: arena_circle.x,
                y: arena_circle.y,
                z: 0.0,
                is_sprite: true,
                sprite: 70,
                sprite_scale: scale,
                weapon_names: None,
                first_spawn_time: None,
                spawn_time: None,
                spawn_timer: None,
                ammo: None,
                hitbox: Hitbox::new(
                    2.0*sprite_scale_mult * scale,
                    2.0*sprite_scale_mult * scale,
                    0.0,
                    HitboxShape::Circle,
                ),
                effects: None,
            })
            .build();        
        
        if arena_circle.obstacle_type == ObstacleType::Wall {
            //setup nav mesh grid
            let offset = sprite_scale_mult*scale + nav_mesh_offset;

            let xr_minus = (arena_circle.x - offset).round();
            let xr_plus = (arena_circle.x + offset).round();

            let xr_minus_find_index = nav_mesh_grid_xs.iter().position(|&r| r == xr_minus);
            let xr_plus_find_index = nav_mesh_grid_xs.iter().position(|&r| r == xr_plus);

            if xr_minus_find_index.is_none() {
                nav_mesh_grid_xs.push(xr_minus);
            }
            if xr_plus_find_index.is_none() {
                nav_mesh_grid_xs.push(xr_plus);
            }


            let yr_minus = (arena_circle.y - offset).round();
            let yr_plus = (arena_circle.y + offset).round();

            let yr_minus_find_index = nav_mesh_grid_ys.iter().position(|&r| r == yr_minus);
            let yr_plus_find_index = nav_mesh_grid_ys.iter().position(|&r| r == yr_plus);

            if yr_minus_find_index.is_none() {
                nav_mesh_grid_ys.push(yr_minus);
            }
            if yr_plus_find_index.is_none() {
                nav_mesh_grid_ys.push(yr_plus);
            }


            nav_mesh_grid_drop.push((xr_minus, xr_plus, yr_minus, yr_plus));
        }
    }


    //Arena King Hill
    for king_hill in arena_properties.king_hills.iter() {
        let sprite_scale_mult = 10.0;
        let scale = king_hill.radius / sprite_scale_mult;

        let mut circle_transform = Transform::default();

        circle_transform.set_translation_xyz(king_hill.x, king_hill.y, -0.02);
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
            .with(ArenaElement {
                obstacle_type: ObstacleType::Open,
                is_hill: true,
                checkpoint: RaceCheckpointType::NotCheckpoint,
                checkpoint_id: 0,
                is_weapon_box: false,
                is_spawn_point: false,
                is_weapon_spawn_point: false,
                x: king_hill.x,
                y: king_hill.y,
                z: 0.0,
                is_sprite: true,
                sprite: 29,
                sprite_scale: scale,
                weapon_names: None,
                first_spawn_time: None,
                spawn_time: None,
                spawn_timer: None,
                ammo: None,
                hitbox: Hitbox::new(
                    20.0 * scale,
                    20.0 * scale,
                    0.0,
                    HitboxShape::Circle,
                ),
                effects: None,
            })
            .with(Transparent)
            .with(king_tint)
            .build();
    }


    //Race Checkpoints and Finish Lines
    for (idx, race_checkpoint) in arena_properties.race_checkpoints.iter().enumerate() {
        let scale = race_checkpoint.length / 20.0;

        let checkpoint_line_sprite_render;
        let checkpoint_type;

        if idx == 0 { //checkered finish line
            checkpoint_type = RaceCheckpointType::Lap;

            checkpoint_line_sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: 30,
            };
        }
        else { //solid white checkpoint line
            checkpoint_type = RaceCheckpointType::Checkpoint;

            checkpoint_line_sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: 31,
            };
        }
        
        let mut checkpoint_line_transform = Transform::default();

        checkpoint_line_transform.set_rotation_2d(race_checkpoint.rotation/180.0 * PI);
        checkpoint_line_transform.set_translation_xyz(race_checkpoint.x, race_checkpoint.y, -0.02);
        checkpoint_line_transform.set_scale(Vector3::new(scale, scale, 0.0));

        
        let width = (20.0 * scale * (race_checkpoint.rotation/180.0*PI).cos().abs())
            + (2.0 * scale * (race_checkpoint.rotation/180.0*PI).sin().abs());
        let height = (2.0 * scale * (race_checkpoint.rotation/180.0*PI).cos().abs()) 
            + (20.0 * scale * (race_checkpoint.rotation/180.0*PI).sin().abs());

        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(checkpoint_line_transform)
            .with(checkpoint_line_sprite_render)
            .with(ArenaElement {
                obstacle_type: ObstacleType::Open,
                is_hill: false,
                checkpoint: checkpoint_type,
                checkpoint_id: idx as i32,
                is_weapon_box: false,
                is_spawn_point: false,
                is_weapon_spawn_point: false,
                x: race_checkpoint.x,
                y: race_checkpoint.y,
                z: 0.0,
                is_sprite: true,
                sprite: 31,
                sprite_scale: scale,
                weapon_names: None,
                first_spawn_time: None,
                spawn_time: None,
                spawn_timer: None,
                ammo: None,
                hitbox: Hitbox::new(
                    width,
                    height,
                    0.0,
                    HitboxShape::Rectangle,
                ),
                effects: None,
            })
            .build();
    }


    //Add non-mesh Arena items

    for weapon_spawner in arena_properties.weapon_spawners.iter() {
        world
            .create_entity()
            .with(Removal::new(0 as u32))
            .with(reform_weapon_spawner((*weapon_spawner).clone()))
            .build();
    }



    //Build navigation mesh from grid
    let mut nav_mesh_vertices: Vec<(f32, f32, f32)> = Vec::new();
    let mut nav_mesh_triangles: Vec<(usize, usize, usize)> = Vec::new();

    //Filter and sort
    nav_mesh_grid_xs.retain(|&x| x>= 0.0 && x<= arena_properties.width);
    nav_mesh_grid_xs.sort_by(|a, b| a.partial_cmp(b).unwrap());

    nav_mesh_grid_ys.retain(|&y| y>= 0.0 && y<= arena_properties.height);
    nav_mesh_grid_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let xs_len = nav_mesh_grid_xs.len();
    let ys_len = nav_mesh_grid_xs.len();

    log::info!("{} {}", xs_len, ys_len);
    log::info!("{:?} {:?}", nav_mesh_grid_xs, nav_mesh_grid_ys);
    log::info!("{:?}", nav_mesh_grid_drop);

    //if *x >= 0.0 && *x <= arena_properties.width {
    //if *y >= 0.0 && *y <= arena_properties.height {

    for (y_idx, y) in nav_mesh_grid_ys.iter().enumerate() {
        for (x_idx, x) in nav_mesh_grid_xs.iter().enumerate() {
            nav_mesh_vertices.push((*x, *y, debug_line_z));

            let vertex_idx = nav_mesh_vertices.len() - 1;

            if (x_idx > 0) && (y_idx > 0) { //don't evaluate first index, otherwise underflow
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