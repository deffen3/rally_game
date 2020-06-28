use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    utils::removal::Removal,
};

use rand::Rng;
use std::f32::consts::PI;

use crate::resources::{WeaponFireResource, GameModes};

use crate::components::{Hitbox, HitboxShape, RaceCheckpointType};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH};

pub fn spawn_weapon_boxes(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    lazy_update: &ReadExpect<LazyUpdate>,
    weapon_box_count: u32,
    game_mode_setup: GameModes, 
) {
    let mut rng = rand::thread_rng();
    let mut spawn_index;

    let mut previous_indices = vec![];

    for _idx in 0..weapon_box_count {
        spawn_index = rng.gen_range(0, 4) as u32;

        while previous_indices.contains(&spawn_index) {
            spawn_index = rng.gen_range(0, 4) as u32;
        }

        let box_entity: Entity = entities.create();

        let mut local_transform = Transform::default();

        if game_mode_setup == GameModes::Race {
            let spacing_factor = 5.0;

            let (x, y) = match spawn_index {
                0 => (
                    ARENA_WIDTH / spacing_factor,
                    ARENA_HEIGHT / spacing_factor,
                ),
                1 => (
                    ARENA_WIDTH - (ARENA_WIDTH / spacing_factor),
                    ARENA_HEIGHT - (ARENA_HEIGHT / spacing_factor),
                ),
                2 => (
                    ARENA_WIDTH / spacing_factor,
                    ARENA_HEIGHT - (ARENA_HEIGHT / spacing_factor),
                ),
                3 => (
                    ARENA_WIDTH - (ARENA_WIDTH / spacing_factor),
                    ARENA_HEIGHT / spacing_factor,
                ),
                _ => (
                    ARENA_WIDTH / spacing_factor,
                    ARENA_HEIGHT / spacing_factor,
                ),
            };

            local_transform.set_translation_xyz(x, y, 0.3);
        }
        else {
            let spacing_factor;
            if game_mode_setup == GameModes::KingOfTheHill {
                spacing_factor = 3.3;
            }
            else {
                spacing_factor = 3.0;
            }
            

            let (x, y) = match spawn_index {
                0 => (ARENA_WIDTH / spacing_factor, ARENA_HEIGHT / 2.0),
                1 => (ARENA_WIDTH / 2.0, ARENA_HEIGHT / spacing_factor),
                2 => (
                    ARENA_WIDTH - (ARENA_WIDTH / spacing_factor),
                    ARENA_HEIGHT / 2.0,
                ),
                3 => (
                    ARENA_WIDTH / 2.0,
                    ARENA_HEIGHT - (ARENA_HEIGHT / spacing_factor),
                ),
                _ => (
                    ARENA_WIDTH / spacing_factor,
                    ARENA_HEIGHT / spacing_factor,
                ),
            };

            local_transform.set_translation_xyz(x, y, 0.3);
        }
        
        local_transform.set_rotation_2d(PI / 8.0);

        let box_sprite = weapon_fire_resource.weapon_box_sprite_render.clone();

        lazy_update.insert(
            box_entity,
            Hitbox::new(
                11.0,
                11.0,
                0.0,
                HitboxShape::Rectangle,
                false,
                false,
                RaceCheckpointType::NotCheckpoint,
                0,
                true,
            ),
        );
        lazy_update.insert(box_entity, Removal::new(0 as u32));
        lazy_update.insert(box_entity, box_sprite);
        lazy_update.insert(box_entity, local_transform);

        previous_indices.push(spawn_index.clone());
    }
}
