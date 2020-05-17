use amethyst::{
    core::math::Vector3,
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    utils::removal::Removal,
};

use rand::Rng;
use std::f32::consts::PI;

use crate::resources::WeaponFireResource;

use crate::components::{Particles};


pub fn malfunction_sparking(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    position: Vector3<f32>,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let sparks_entity: Entity = entities.create();

    let sparks_sprite = weapon_fire_resource.sparking_sprite_render.clone();

    let mut local_transform = Transform::default();
    local_transform.set_translation(position);


    let mut rng = rand::thread_rng();
    let random_rotation_angle = rng.gen_range(-PI, PI);

    local_transform.set_rotation_2d(random_rotation_angle);

    let random_velocity_angle = rng.gen_range(-PI, PI);

    let x_comp = -random_velocity_angle.sin();
    let y_comp = random_velocity_angle.cos();

    let velocity = rng.gen_range(15.0, 30.0);

    lazy_update.insert(sparks_entity, Particles {
        dx: velocity * x_comp,
        dy: velocity * y_comp,
        life_timer: 0.2,
    });
    
    lazy_update.insert(sparks_entity, sparks_sprite);
    lazy_update.insert(sparks_entity, local_transform);

    lazy_update.insert(sparks_entity, Removal::new(0 as u32));
}