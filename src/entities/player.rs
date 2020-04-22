use amethyst::{
    core::transform::Transform,
    assets::{Handle},
    renderer::{SpriteRender, SpriteSheet},
    ecs::prelude::{World, Entity, Entities},
    prelude::*,
};

use std::f32::consts::PI;

use crate::components::{
    Player, Vehicle, Weapon, WeaponFire, WeaponTypes,
};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH};

pub fn intialize_player(
    world: &mut World, 
    sprite_sheet_handle: Handle<SpriteSheet>,
    player_index: usize,
    weapon_type: WeaponTypes,
) {
let mut vehicle_transform = Transform::default();

let (starting_rotation, starting_x, starting_y) = match player_index {
    0 => (-PI/4.0, ARENA_WIDTH / 5.0, ARENA_HEIGHT / 5.0),
    1 => (PI + PI/4.0, ARENA_WIDTH / 5.0, ARENA_HEIGHT - (ARENA_HEIGHT / 5.0)),
    2 => (PI/2.0 - PI/4.0, ARENA_WIDTH - (ARENA_WIDTH / 5.0), ARENA_HEIGHT / 5.0),
    3 => (PI/2.0 + PI/4.0, ARENA_WIDTH - (ARENA_WIDTH / 5.0), ARENA_HEIGHT - (ARENA_HEIGHT / 5.0)),
    _ => (-PI/4.0, ARENA_WIDTH / 5.0, ARENA_HEIGHT / 5.0),
};

vehicle_transform.set_rotation_2d(starting_rotation as f32);
vehicle_transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.0);
//vehicle_transform.set_translation_xyz(ARENA_WIDTH / 5.0 * ((player_index + 1) as f32), ARENA_HEIGHT /2.0, 0.0);

let vehicle_sprite_render = SpriteRender {
    sprite_sheet: sprite_sheet_handle.clone(),
    sprite_number: player_index,
};

let (weapon_type,
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
    .with(vehicle_sprite_render)
    .with(Vehicle::new())
    .with(Weapon::new(weapon_type,
        weapon_cooldown, 
        burst_shot_limit,
        burst_cooldown,
        weapon_shot_speed,
        damage,
        shield_damage_pct,
        armor_damage_pct,
        piercing_damage_pct,
        health_damage_pct))
    .with(Player::new(player_index))
    .build();


//I can build all of this as one entity, but then I get only one sprite.
//if I separate it into three entities, then now my systems are broken as their
//  is no relationship between these entities. Do I need to apply parent child relationships?
//  Isn't this going against the purpose/elegance of ECS?
}




fn build_standard_weapon(weapon_type: WeaponTypes) -> (
WeaponTypes, f32, u32, f32, f32, f32, f32, f32, f32, f32
) {
let (weapon_shot_speed, damage, weapon_cooldown, 
        piercing_damage_pct, 
        shield_damage_pct, armor_damage_pct, 
        health_damage_pct,
    ) = match weapon_type.clone()
{                                      //speed      dmg     cooldwn pierce% shield%   armor%    health%
    WeaponTypes::LaserDouble =>         (400.0,     25.0,   0.4,    0.0,   120.0,     75.0,     100.0),
    WeaponTypes::LaserBeam =>           (2800.0,    0.3,    0.0,    0.0,   120.0,     75.0,     100.0),
    WeaponTypes::LaserPulse =>          (400.0,     12.0,   0.75,   0.0,   120.0,     75.0,     100.0),
    WeaponTypes::ProjectileBurstFire => (250.0,     12.0,   0.15,   0.0,    80.0,     90.0,     100.0),
    WeaponTypes::ProjectileRapidFire => (250.0,     3.0,    0.10,   0.0,    80.0,     90.0,     100.0),
    WeaponTypes::ProjectileCannonFire =>(700.0,     50.0,   0.9,    0.0,    80.0,     90.0,     100.0),
    WeaponTypes::Missile =>             (100.0,     50.0,   2.5,    10.0,   75.0,     75.0,     100.0),
    WeaponTypes::Rockets =>             (250.0,     50.0,   0.5,    10.0,   75.0,     75.0,     100.0),
    WeaponTypes::Mine =>                (0.0,       50.0,   2.5,    10.0,   75.0,     75.0,     100.0),
};

let burst_cooldown;
let burst_shot_limit; 
if weapon_type.clone() == WeaponTypes::LaserPulse {
    burst_cooldown = 0.1 as f32;
    burst_shot_limit = 2 as u32;
}
else if weapon_type.clone() == WeaponTypes::ProjectileBurstFire{
    burst_cooldown = 0.1 as f32;
    burst_shot_limit = 2 as u32;
}
else {
    burst_cooldown = weapon_cooldown.clone();
    burst_shot_limit = 1 as u32;
};

(weapon_type,
    weapon_cooldown, 
    burst_shot_limit,
    burst_cooldown,
    weapon_shot_speed,
    damage,
    shield_damage_pct,
    armor_damage_pct,
    piercing_damage_pct,
    health_damage_pct,)
}
