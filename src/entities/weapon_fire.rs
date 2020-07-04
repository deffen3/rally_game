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
    player_id: Option<usize>,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let fire_entity: Entity = entities.create();

    let weapon_array_id = spawner_weapon_fire.weapon_array_id.clone();

    let mut weapon_fire = WeaponFire::new(
        player_id,
        weapon_array_id,
        spawner_weapon_fire.weapon_name.clone(),
        spawner_weapon_fire.weapon_fire_type,
        spawner_weapon_fire.stats,
    );

    weapon_fire.chain_hit_ids = spawner_weapon_fire.chain_hit_ids.clone();

    let local_transform = {
        let mut local_transform = Transform::default();
        local_transform.set_translation(fire_position);

        let angle_x_comp: f32 = -fire_angle.sin();
        let angle_y_comp: f32 = fire_angle.cos();

        local_transform.set_rotation_2d(fire_angle);

        weapon_fire.dx = weapon_fire.stats.shot_speed * angle_x_comp;
        weapon_fire.dy = weapon_fire.stats.shot_speed * angle_y_comp;

        local_transform
    };
    lazy_update.insert(fire_entity, weapon_fire);

    let (_icon_scale, weapon_sprite) =
        get_weapon_icon(player_id, spawner_weapon_fire.weapon_fire_type, weapon_fire_resource);

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
    player_id: Option<usize>,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let fire_entity: Entity = entities.create();

    let mut weapon_fire = WeaponFire::new(
        player_id,
        weapon_array_id,
        weapon.name.clone(),
        weapon.stats.weapon_fire_type,
        weapon.stats.fire_stats,
    );

    let local_transform = {
        let mut local_transform = Transform::default();
        local_transform.set_translation(fire_position);

        let angle_x_comp: f32 = -fire_angle.sin();
        let angle_y_comp: f32 = fire_angle.cos();

        local_transform.set_rotation_2d(fire_angle);

        weapon_fire.dx = weapon_fire.stats.shot_speed * angle_x_comp;
        weapon_fire.dy = weapon_fire.stats.shot_speed * angle_y_comp;

        //adjust the first postion
        let x = local_transform.translation().x;
        let y = local_transform.translation().y;

        let x_offset = 0.0;
        let y_offset = 0.0;

        local_transform.set_translation_x(x - x_offset);
        local_transform.set_translation_y(y + y_offset);

        local_transform
    };
    lazy_update.insert(fire_entity, weapon_fire);

    let (_icon_scale, weapon_sprite) =
        get_weapon_icon(player_id, weapon.stats.weapon_fire_type, weapon_fire_resource);

    lazy_update.insert(fire_entity, weapon_sprite);
    lazy_update.insert(fire_entity, local_transform);

    lazy_update.insert(fire_entity, Removal::new(0 as u32));
}
