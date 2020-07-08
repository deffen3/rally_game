use amethyst::{
    core::math::Vector3,
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    renderer::Transparent,
    utils::removal::Removal,
};

use rand::Rng;
use std::f32::consts::PI;

use crate::resources::WeaponFireResource;

use crate::components::{Particles, Shockwave};

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

    lazy_update.insert(
        sparks_entity,
        Particles {
            dx: velocity * x_comp,
            dy: velocity * y_comp,
            life_timer: 0.2,
        },
    );
    lazy_update.insert(sparks_entity, sparks_sprite);
    lazy_update.insert(sparks_entity, local_transform);

    lazy_update.insert(sparks_entity, Removal::new(0 as u32));
}

pub fn hit_spray(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    shields: bool,
    position: Vector3<f32>,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let spray_entity: Entity = entities.create();

    let spray_sprite;
    if shields {
        spray_sprite = weapon_fire_resource.shield_hit_spray_sprite_render.clone();
    } else {
        spray_sprite = weapon_fire_resource.hull_hit_spray_sprite_render.clone();
    }
    let mut local_transform = Transform::default();
    local_transform.set_translation(position);

    let mut rng = rand::thread_rng();
    let random_rotation_angle = rng.gen_range(-PI, PI);

    local_transform.set_rotation_2d(random_rotation_angle);

    let random_velocity_angle = rng.gen_range(-PI, PI);

    let x_comp = -random_velocity_angle.sin();
    let y_comp = random_velocity_angle.cos();

    let velocity = rng.gen_range(25.0, 40.0);

    lazy_update.insert(
        spray_entity,
        Particles {
            dx: velocity * x_comp,
            dy: velocity * y_comp,
            life_timer: 0.2,
        },
    );
    lazy_update.insert(spray_entity, spray_sprite);
    lazy_update.insert(spray_entity, local_transform);

    lazy_update.insert(spray_entity, Removal::new(0 as u32));
}

pub fn acceleration_spray(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    is_smoking: bool,
    position: Vector3<f32>,
    angle: f32,
    thrust: f32,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let particles_entity: Entity = entities.create();

    let particles_sprite;
    if is_smoking {
        particles_sprite = weapon_fire_resource.smoke_spray_sprite_render.clone();
    } else {
        particles_sprite = weapon_fire_resource.rocket_spray_sprite_render.clone();
    }

    let mut local_transform = Transform::default();
    local_transform.set_translation(position);

    local_transform.set_rotation_2d(angle - PI);

    let mut rng = rand::thread_rng();
    let random_velocity_angle = rng.gen_range(-PI / 6., PI / 6.);

    let spray_angle = angle + random_velocity_angle;

    let x_comp = -spray_angle.sin();
    let y_comp = spray_angle.cos();

    lazy_update.insert(
        particles_entity,
        Particles {
            dx: thrust / 100.0 * x_comp,
            dy: thrust / 100.0 * y_comp,
            life_timer: 0.2,
        },
    );
    lazy_update.insert(particles_entity, particles_sprite);
    lazy_update.insert(particles_entity, Transparent);
    lazy_update.insert(particles_entity, local_transform);

    lazy_update.insert(particles_entity, Removal::new(0 as u32));
}

pub fn explosion_shockwave(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    position: Vector3<f32>,
    radius: f32,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let shockwave_entity: Entity = entities.create();

    let shockwave_sprite = weapon_fire_resource.shockwave_sprite_render.clone();

    let mut local_transform = Transform::default();
    local_transform.set_translation(position);
    local_transform.set_scale(Vector3::new(0.0, 0.0, 0.0));

    let life_time = 0.2;

    lazy_update.insert(
        shockwave_entity,
        Particles {
            dx: 0.0,
            dy: 0.0,
            life_timer: life_time,
        },
    );

    lazy_update.insert(
        shockwave_entity,
        Shockwave {
            radius: radius,
            time: life_time,
        },
    );
    lazy_update.insert(shockwave_entity, shockwave_sprite);
    lazy_update.insert(shockwave_entity, local_transform);
    lazy_update.insert(shockwave_entity, Transparent);

    lazy_update.insert(shockwave_entity, Removal::new(0 as u32));
}
