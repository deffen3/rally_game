use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    utils::removal::Removal,
};

use rand::Rng;
use std::f32::consts::PI;

use crate::resources::{WeaponFireResource};

use crate::components::{
    WeaponSpawnBox, reform_weapon_spawn_box, 
    ArenaProperties,
};


pub fn spawn_weapon_boxes(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    lazy_update: &ReadExpect<LazyUpdate>,
    weapon_box_count: u32,
    arena_properties: &ArenaProperties, 
) {
    let mut rng = rand::thread_rng();
    let mut spawn_index;

    let mut previous_indices = vec![];

    let number_of_spawn_locations = arena_properties.weapon_spawn_boxes.len();

    for _idx in 0..weapon_box_count.min(number_of_spawn_locations as u32) {
        
        spawn_index = rng.gen_range(0, number_of_spawn_locations) as usize;

        while previous_indices.contains(&spawn_index) {
            spawn_index = rng.gen_range(0, number_of_spawn_locations) as usize;
        }

        let spawn_box = arena_properties.weapon_spawn_boxes[spawn_index];

        let box_entity: Entity = entities.create();

        let mut local_transform = Transform::default();
        
        local_transform.set_rotation_2d(PI / 8.0);

        let box_sprite = weapon_fire_resource.weapon_box_sprite_render.clone();

        let weapon_spawn_box = WeaponSpawnBox {x: spawn_box.x, y: spawn_box.y, weapon_name: spawn_box.weapon_name};

        lazy_update.insert(box_entity, reform_weapon_spawn_box(weapon_spawn_box));
        lazy_update.insert(box_entity, Removal::new(0 as u32));
        lazy_update.insert(box_entity, box_sprite);
        lazy_update.insert(box_entity, local_transform);

        previous_indices.push(spawn_index.clone());
    }
}
