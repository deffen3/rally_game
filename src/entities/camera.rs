use amethyst::{
    core::transform::{Parent, Transform},
    ecs::prelude::{Entity, World},
    prelude::*,
    renderer::Camera,
};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH};

pub fn initialize_camera(world: &mut World) -> Entity {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build()
}

pub fn initialize_camera_to_player(world: &mut World, parent: Entity) -> Entity {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
    let mut transform = Transform::default();
    transform.set_translation_z(1.0);

    world
        .create_entity()
        .with(Parent { entity: parent })
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build()
}
