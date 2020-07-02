use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    utils::removal::Removal,
};

use rand::Rng;
use std::f32::consts::PI;

use crate::resources::{WeaponFireResource, GameWeaponSetup};

use crate::components::{
    reform_weapon_spawn_box, WeaponSpawnBox,
    ArenaProperties,
};


pub fn spawn_random_weapon_boxes(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    lazy_update: &ReadExpect<LazyUpdate>,
    game_weapon_setup: &GameWeaponSetup,
    arena_properties: &ArenaProperties,
) {
    let mut rng = rand::thread_rng();
    let mut spawn_index;

    let mut previous_indices = vec![];

    //Check if need to filter out the special map-defined spawn boxes
    let mut random_weapon_spawn_boxes: Vec<WeaponSpawnBox> = Vec::new();
    for spawn_box in arena_properties.weapon_spawn_boxes.iter() {
        if game_weapon_setup.allow_map_specific_spawn_weapons && spawn_box.weapon_name == None {
            random_weapon_spawn_boxes.push((*spawn_box).clone());
        }
    }

    let number_of_random_spawn_locations = random_weapon_spawn_boxes.len();

    for _idx in 0..game_weapon_setup.random_weapon_spawn_count.min(number_of_random_spawn_locations as u32) {
        
        spawn_index = rng.gen_range(0, number_of_random_spawn_locations) as usize;

        while previous_indices.contains(&spawn_index) {
            spawn_index = rng.gen_range(0, number_of_random_spawn_locations) as usize;
        }

        let spawn_box = random_weapon_spawn_boxes[spawn_index].clone();

        let box_entity: Entity = entities.create();

        let mut local_transform = Transform::default();
        
        local_transform.set_rotation_2d(PI / 8.0);
        local_transform.set_translation_xyz(spawn_box.x, spawn_box.y, 0.3);

        let box_sprite = weapon_fire_resource.weapon_box_sprite_render.clone();

        lazy_update.insert(box_entity, reform_weapon_spawn_box(spawn_box));
        lazy_update.insert(box_entity, Removal::new(0 as u32));
        lazy_update.insert(box_entity, box_sprite);
        lazy_update.insert(box_entity, local_transform);

        previous_indices.push(spawn_index.clone());
    }
}