use amethyst::{
    core::math::Vector3,
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    utils::removal::Removal,
};

use crate::resources::WeaponFireResource;

use crate::components::{get_weapon_icon, Weapon, WeaponFire};



pub fn chain_fire_weapon(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    spawner_weapon_fire: WeaponFire,
    fire_position: Vector3<f32>,
    fire_angle: f32,
    player_id: usize,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let fire_entity: Entity = entities.create();

    let weapon_array_id = spawner_weapon_fire.weapon_array_id.clone();

    let mut weapon_fire = WeaponFire::new(
        weapon_array_id,
        spawner_weapon_fire.weapon_name.clone(),
        spawner_weapon_fire.weapon_type,
        player_id,
        spawner_weapon_fire.heat_seeking,
        spawner_weapon_fire.heat_seeking_agility,
        spawner_weapon_fire.attached,
        spawner_weapon_fire.deployed,
        spawner_weapon_fire.spawn_angle,
        spawner_weapon_fire.shot_speed,
        spawner_weapon_fire.accel_rate,
        spawner_weapon_fire.shot_life_limit,
        spawner_weapon_fire.damage,
        spawner_weapon_fire.trigger_radius,
        spawner_weapon_fire.trigger_immediately,
        spawner_weapon_fire.damage_radius,
        spawner_weapon_fire.shield_damage_pct,
        spawner_weapon_fire.armor_damage_pct,
        spawner_weapon_fire.piercing_damage_pct,
        spawner_weapon_fire.health_damage_pct,
        spawner_weapon_fire.ion_malfunction_pct,
        spawner_weapon_fire.duration_damage,
        spawner_weapon_fire.bounces,
        spawner_weapon_fire.chaining_damage,
        spawner_weapon_fire.slow_down_effect,
        spawner_weapon_fire.stuck_accel_effect_timer,
    );

    let local_transform = {
        let mut local_transform = Transform::default();
        local_transform.set_translation(fire_position);

        let angle_x_comp: f32 = -fire_angle.sin();
        let angle_y_comp: f32 = fire_angle.cos();

        local_transform.set_rotation_2d(fire_angle);

        weapon_fire.dx = weapon_fire.shot_speed * angle_x_comp;
        weapon_fire.dy = weapon_fire.shot_speed * angle_y_comp;

        local_transform
    };
    lazy_update.insert(fire_entity, weapon_fire);

    let (_icon_scale, weapon_sprite) =
        get_weapon_icon(player_id, spawner_weapon_fire.weapon_type, weapon_fire_resource);

    lazy_update.insert(fire_entity, weapon_sprite);
    lazy_update.insert(fire_entity, local_transform);

    lazy_update.insert(fire_entity, Removal::new(0 as u32));
}


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

        //let x_offset = weapon_fire.height*0.5 * angle_x_comp + weapon_fire.width*0.5 * (1.0-angle_x_comp);
        //let y_offset = weapon_fire.height*0.5 * angle_y_comp + weapon_fire.width*0.5 * (1.0-angle_y_comp);
        let x_offset = 0.0;
        let y_offset = 0.0;

        local_transform.set_translation_x(x - x_offset);
        local_transform.set_translation_y(y + y_offset);

        local_transform
    };
    lazy_update.insert(fire_entity, weapon_fire);

    let (_icon_scale, weapon_sprite) =
        get_weapon_icon(player_id, weapon.stats.weapon_type, weapon_fire_resource);

    lazy_update.insert(fire_entity, weapon_sprite);
    lazy_update.insert(fire_entity, local_transform);

    lazy_update.insert(fire_entity, Removal::new(0 as u32));
}
