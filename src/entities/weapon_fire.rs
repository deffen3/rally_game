use amethyst::{
    core::math::Vector3,
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    utils::removal::Removal,
};

use rand::Rng;
use std::f32::consts::PI;

use crate::resources::WeaponFireResource;

use crate::components::{get_weapon_icon, Weapon, WeaponFire, Sparks};


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

    lazy_update.insert(sparks_entity, Sparks {
        dx: velocity * x_comp,
        dy: velocity * y_comp,
        life_timer: 0.2,
    });
    
    lazy_update.insert(sparks_entity, sparks_sprite);
    lazy_update.insert(sparks_entity, local_transform);

    lazy_update.insert(sparks_entity, Removal::new(0 as u32));
}


pub fn fire_weapon(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    weapon: Weapon,
    fire_position: Vector3<f32>,
    fire_angle: f32,
    player_id: usize,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let fire_entity: Entity = entities.create();

    let mut weapon_fire = WeaponFire::new(
        weapon.name.clone(),
        weapon.stats.weapon_type,
        player_id,
        weapon.stats.heat_seeking,
        weapon.stats.heat_seeking_agility,
        weapon.stats.attached,
        weapon.stats.deployed,
        weapon.stats.mounted_angle,
        weapon.stats.shot_speed,
        weapon.stats.shot_life_limit,
        weapon.stats.damage,
        weapon.stats.shield_damage_pct,
        weapon.stats.armor_damage_pct,
        weapon.stats.piercing_damage_pct,
        weapon.stats.health_damage_pct,
    );

    let local_transform = {
        let mut local_transform = Transform::default();
        local_transform.set_translation(fire_position);

        let angle_x_comp: f32 = -fire_angle.sin();
        let angle_y_comp: f32 = fire_angle.cos();

        local_transform.set_rotation_2d(fire_angle);

        weapon_fire.dx = weapon_fire.shot_speed * angle_x_comp;
        weapon_fire.dy = weapon_fire.shot_speed * angle_y_comp;

        //adjust the first postion
        let x = local_transform.translation().x;
        let y = local_transform.translation().y;

        //let yaw_width = weapon_fire.height*0.5 * angle_x_comp + weapon_fire.width*0.5 * (1.0-angle_x_comp);
        //let yaw_height = weapon_fire.height*0.5 * angle_y_comp + weapon_fire.width*0.5 * (1.0-angle_y_comp);
        let yaw_width = 0.0;
        let yaw_height = 0.0;

        local_transform.set_translation_x(x - yaw_width);
        local_transform.set_translation_y(y + yaw_height);

        local_transform
    };
    lazy_update.insert(fire_entity, weapon_fire);

    let (_icon_scale, weapon_sprite) =
        get_weapon_icon(player_id, weapon.stats, weapon_fire_resource);

    lazy_update.insert(fire_entity, weapon_sprite);
    lazy_update.insert(fire_entity, local_transform);

    lazy_update.insert(fire_entity, Removal::new(0 as u32));
}
