use amethyst::{
    core::transform::Transform,
    renderer::{Camera},
    ecs::prelude::{World, Entity, Entities},
    prelude::*,
};

use crate::rally::{ARENA_WIDTH, ARENA_HEIGHT};

pub fn initialise_camera(world: &mut World) {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left. 
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();
}
