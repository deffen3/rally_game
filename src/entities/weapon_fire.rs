use amethyst::{
    core::math::Vector3,
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    utils::removal::Removal,
};

use crate::resources::WeaponFireResource;

use crate::components::{get_weapon_icon, Weapon, WeaponFire};


pub fn fire_weapon(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    weapon: Weapon,
    weapon_array_id: usize,
    fire_position: Vector3<f32>,
    fire_angle: f32,
    player_id: usize,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let fire_entity: Entity = entities.create();

    let mut weapon_fire = WeaponFire::new(
        weapon_array_id,
        weapon.name.clone(),
        weapon.stats.weapon_type,
        player_id,
        weapon.stats.heat_seeking,
        weapon.stats.heat_seeking_agility,
        weapon.stats.attached,
        weapon.stats.deployed,
        weapon.stats.mounted_angle,
        weapon.stats.shot_speed,
        weapon.stats.accel_rate,
        weapon.stats.shot_life_limit,
        weapon.stats.damage,
        weapon.stats.trigger_radius,
        weapon.stats.trigger_immediately,
        weapon.stats.damage_radius,
        weapon.stats.shield_damage_pct,
        weapon.stats.armor_damage_pct,
        weapon.stats.piercing_damage_pct,
        weapon.stats.health_damage_pct,
        weapon.stats.ion_malfunction_pct,
        weapon.stats.duration_damage,
        weapon.stats.bounces,
        weapon.stats.chaining_damage,
        weapon.stats.slow_down_effect,
        weapon.stats.stuck_accel_effect_timer,
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
