use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    utils::removal::Removal,
};

use std::f32::consts::PI;

use crate::resources::WeaponFireResource;

use crate::components::ArenaElement;

pub fn spawn_weapon_box_from_spawner(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    lazy_update: &ReadExpect<LazyUpdate>,
    weapon_spawner: &ArenaElement,
) {
    let box_entity: Entity = entities.create();
    let mut local_transform = Transform::default();
    local_transform.set_rotation_2d(PI / 8.0);
    local_transform.set_translation_xyz(weapon_spawner.x, weapon_spawner.y, 0.3);

    let box_sprite = weapon_fire_resource.weapon_box_sprite_render.clone();

    let mut spawn_box_arena_element = weapon_spawner.clone();
    spawn_box_arena_element.is_weapon_spawn_point = false;
    spawn_box_arena_element.is_weapon_box = true;

    lazy_update.insert(box_entity, spawn_box_arena_element);
    lazy_update.insert(box_entity, Removal::new(0 as u32));
    lazy_update.insert(box_entity, box_sprite);
    lazy_update.insert(box_entity, local_transform);
}
