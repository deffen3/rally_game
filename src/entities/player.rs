use amethyst::{
    core::transform::Transform,
    assets::{Handle},
    renderer::{SpriteRender, SpriteSheet},
    ecs::prelude::{World},
    prelude::*,
};

use amethyst::core::math::Vector3;
use std::f32::consts::PI;

use crate::components::{
    Player, Vehicle, Weapon, WeaponTypes, build_standard_weapon, PlayerWeaponIcon,
};
use crate::resources::{WeaponFireResource};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};

pub fn intialize_player(
    world: &mut World, 
    sprite_sheet_handle: Handle<SpriteSheet>,
    player_index: usize,
    weapon_type: WeaponTypes,
    weapon_fire_resource: WeaponFireResource,
    is_bot: bool,
) {
    let mut vehicle_transform = Transform::default();

    let spacing_factor = 5.0;

    let height = ARENA_HEIGHT + UI_HEIGHT;

    let (starting_rotation, starting_x, starting_y) = match player_index {
        0 => (-PI/4.0, ARENA_WIDTH / spacing_factor, height / spacing_factor),
        1 => (PI/2.0 + PI/4.0, ARENA_WIDTH - (ARENA_WIDTH / spacing_factor), height - (height / spacing_factor)),
        2 => (PI + PI/4.0, ARENA_WIDTH / spacing_factor, height - (height / spacing_factor)),
        3 => (PI/2.0 - PI/4.0, ARENA_WIDTH - (ARENA_WIDTH / spacing_factor), height / spacing_factor),
        _ => (-PI/4.0, ARENA_WIDTH / spacing_factor, height / spacing_factor),
    };

    vehicle_transform.set_rotation_2d(starting_rotation as f32);
    vehicle_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.0);

    let vehicle_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: player_index,
    };

    let (weapon_type,
        heat_seeking,
        heat_seeking_agility,
        attached,
        deployed,
        weapon_cooldown, 
        burst_shot_limit,
        burst_cooldown,
        weapon_shot_speed,
        damage,
        shield_damage_pct,
        armor_damage_pct,
        piercing_damage_pct,
        health_damage_pct,) = build_standard_weapon(weapon_type);

    world
        .create_entity()
        .with(vehicle_transform)
        .with(vehicle_sprite_render.clone())
        .with(Vehicle::new())
        .with(Weapon::new(weapon_type.clone(),
            heat_seeking,
            heat_seeking_agility,
            attached,
            deployed,
            weapon_cooldown, 
            burst_shot_limit,
            burst_cooldown,
            weapon_shot_speed,
            damage,
            shield_damage_pct,
            armor_damage_pct,
            piercing_damage_pct,
            health_damage_pct))
        .with(Player::new(player_index, is_bot))
        .build();


    //I can build all of this as one entity, but then I get only one sprite.
    //if I separate it into three entities, then now my systems are broken as their
    //  is no relationship between these entities. Do I need to apply parent child relationships?
    //  Isn't this going against the purpose/elegance of ECS?



    //UI vehicle icons
    let x = 20.;
    let y = UI_HEIGHT - 10.;
    let dx = 32.;
    let dx2 = 4.;
    {
        let mut icon_transform = Transform::default();

        let starting_x = match player_index {
            0 => (x),
            1 => (x + 3.0*dx + dx2),
            2 => (x + 6.0*dx + 2.0*dx2),
            3 => (x + 9.0*dx + 3.0*dx2),
            _ => (0.0),
        };

        icon_transform.set_rotation_2d(-PI/2.0);
        icon_transform.set_translation_xyz(starting_x as f32, y, 0.0);

        world
            .create_entity()
            .with(icon_transform)
            .with(vehicle_sprite_render)
            .build();
    }

    //UI initial weapon icon
    let x = 15.;
    let y = UI_HEIGHT - 10.;
    let dx = 32.;
    let dx2 = 4.;

    let weapon_icon_dx = 70.0;

    let (icon_scale, mut weapon_sprite) = match weapon_type.clone() {
        WeaponTypes::LaserDouble => (3.0, weapon_fire_resource.laser_double_sprite_render.clone()),
        WeaponTypes::LaserBeam => (1.0, weapon_fire_resource.laser_beam_sprite_render.clone()),
        WeaponTypes::LaserPulse => (3.0, weapon_fire_resource.laser_burst_sprite_render.clone()),
        WeaponTypes::ProjectileBurstFire => (3.0, weapon_fire_resource.projectile_burst_render.clone()),
        WeaponTypes::ProjectileRapidFire => (3.0, weapon_fire_resource.projectile_rapid_render.clone()),
        WeaponTypes::ProjectileCannonFire => (3.0, weapon_fire_resource.projectile_cannon_sprite_render.clone()),
        WeaponTypes::Missile => (2.0, weapon_fire_resource.missile_sprite_render.clone()),
        WeaponTypes::Rockets => (2.0, weapon_fire_resource.rockets_sprite_render.clone()),
        WeaponTypes::Mine => (2.0, weapon_fire_resource.mine_p1_sprite_render.clone()),
        WeaponTypes::LaserSword => (1.0, weapon_fire_resource.laser_sword_sprite_render.clone()),
    };

    if weapon_type.clone() == WeaponTypes::Mine {
        weapon_sprite = match player_index {
            0 => weapon_fire_resource.mine_p1_sprite_render.clone(),
            1 => weapon_fire_resource.mine_p2_sprite_render.clone(),
            2 => weapon_fire_resource.mine_p3_sprite_render.clone(),
            3 => weapon_fire_resource.mine_p4_sprite_render.clone(),
            _ => weapon_fire_resource.mine_p1_sprite_render.clone(),
        }
    }

    let mut icon_weapon_transform = Transform::default();

    let starting_x = match player_index {
        0 => (x),
        1 => (x + 3.0*dx + dx2),
        2 => (x + 6.0*dx + 2.0*dx2),
        3 => (x + 9.0*dx + 3.0*dx2),
        _ => (0.0),
    };

    icon_weapon_transform.set_translation_xyz((starting_x + weapon_icon_dx) as f32, y, 0.0);
    icon_weapon_transform.set_rotation_2d(-PI/2.0);
    icon_weapon_transform.set_scale(Vector3::new(icon_scale, icon_scale, 0.0));

    world
        .create_entity()
        .with(PlayerWeaponIcon::new(player_index, weapon_type))
        .with(weapon_sprite)
        .with(icon_weapon_transform)
        .build();
}