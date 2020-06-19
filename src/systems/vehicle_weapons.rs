use amethyst::{
    core::{math::Vector3, Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, LazyUpdate, Read, ReadExpect, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::{palette::Srgba, resources::Tint},
};

use rand::Rng;
use std::f32::consts::PI;

use crate::components::{BotMode, Player, Vehicle, VehicleState, WeaponArray, WEAPON_ARRAY_SIZE};
use crate::entities::fire_weapon;
use crate::resources::WeaponFireResource;


#[derive(SystemDesc)]
pub struct VehicleWeaponsSystem;

impl<'s> System<'s> for VehicleWeaponsSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, WeaponArray>,
        WriteStorage<'s, Tint>,
        ReadExpect<'s, WeaponFireResource>,
        ReadExpect<'s, LazyUpdate>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>, //<MovementBindingTypes>>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut players,
            mut transforms,
            mut vehicles,
            mut weapon_arrays,
            mut tints,
            weapon_fire_resource,
            lazy_update,
            time,
            input,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        let dt = time.delta_seconds();

        for (player, vehicle, weapon_array, transform) in
            (&mut players, &mut vehicles, &mut weapon_arrays, &mut transforms).join()
        {
            if vehicle.state == VehicleState::Active {
                //let vehicle_weapon_fire = input.action_is_down(&ActionBinding::VehicleShoot(player.id));

                let mut vehicle_weapon_fire: [Option<bool>; WEAPON_ARRAY_SIZE] = [None; WEAPON_ARRAY_SIZE];
                
                vehicle_weapon_fire[0] = match player.id {
                    0 => input.action_is_down("p1_fire"),
                    1 => input.action_is_down("p2_fire"),
                    2 => input.action_is_down("p3_fire"),
                    3 => input.action_is_down("p4_fire"),
                    _ => None,
                };

                vehicle_weapon_fire[1] = match player.id {
                    0 => input.action_is_down("p1_alt_fire"),
                    1 => input.action_is_down("p2_alt_fire"),
                    2 => input.action_is_down("p3_alt_fire"),
                    3 => input.action_is_down("p4_alt_fire"),
                    _ => None,
                };

                if player.is_bot {
                    if player.bot_mode == BotMode::StopAim
                        || player.bot_mode == BotMode::StrafeAim
                        || player.bot_mode == BotMode::Mining
                        || player.bot_mode == BotMode::Chasing
                        || player.bot_mode == BotMode::Swording
                    {
                        vehicle_weapon_fire[0] = Some(rng.gen::<bool>());
                        vehicle_weapon_fire[1] = Some(rng.gen::<bool>());
                    }
                }

                for (weapon_index, weapon) in weapon_array.weapons.iter_mut().enumerate() {
                    if let Some(weapon) = weapon {
                        if let Some(fire) = vehicle_weapon_fire[weapon_index] {
                            if !vehicle.repair.activated {
                                if fire && weapon.cooldown_timer <= 0.0 {
                                    let vehicle_rotation = transform.rotation();
                                    let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();

                                    let yaw_x_comp = -vehicle_angle.sin(); //left is -, right is +
                                    let yaw_y_comp = vehicle_angle.cos(); //up is +, down is -

                                    let fire_position = Vector3::new(
                                        transform.translation().x + yaw_x_comp * 5.0,
                                        transform.translation().y + yaw_y_comp * 5.0,
                                        0.0,
                                    );

                                    //typical angle this weapon should fire at
                                    let standard_angle = vehicle_angle + weapon.stats.mounted_angle;

                                    //result angle the weapon fires at, after adding any auto-aim or spread modifiers
                                    let mut fire_angle: f32;

                                    if let Some(angle_to_closest_targetable_vehicle) =
                                        vehicle.angle_to_closest_targetable_vehicle
                                    {
                                        let angle_to_selected_vehicle = angle_to_closest_targetable_vehicle;
                                        let dist_to_selected_vehicle =
                                            vehicle.dist_to_closest_targetable_vehicle.unwrap();

                                        fire_angle = calc_tracking_fire_angle(
                                            dist_to_selected_vehicle,
                                            angle_to_selected_vehicle,
                                            standard_angle,
                                            weapon.stats.tracking_angle,
                                        );
                                    } else if let Some(angle_to_closest_targetable_vehicle) =
                                        vehicle.angle_to_closest_targetable_vehicle
                                    {
                                        let angle_to_selected_vehicle = angle_to_closest_targetable_vehicle;
                                        let dist_to_selected_vehicle =
                                            vehicle.dist_to_closest_targetable_vehicle.unwrap();

                                        fire_angle = calc_tracking_fire_angle(
                                            dist_to_selected_vehicle,
                                            angle_to_selected_vehicle,
                                            standard_angle,
                                            weapon.stats.tracking_angle,
                                        );
                                    } else {
                                        fire_angle = standard_angle; //no tracking, no vehicles
                                    }

                                    if weapon.stats.spread_angle >= 0.001 {
                                        let spread_angle_modifier =
                                            rng.gen_range(-1.0, 1.0) * weapon.stats.spread_angle;
                                        fire_angle += spread_angle_modifier;
                                    }

                                    if !weapon.stats.attached || !weapon.stats.deployed {
                                        if !weapon.stats.deployed {
                                            weapon.stats.deployed = true;
                                        }

                                        fire_weapon(
                                            &entities,
                                            &weapon_fire_resource,
                                            weapon.clone(),
                                            fire_position,
                                            fire_angle,
                                            player.id,
                                            &lazy_update,
                                        );
                                    }

                                    if weapon.stats.burst_shot_limit > 0
                                        && weapon.burst_shots < weapon.stats.burst_shot_limit
                                    {
                                        weapon.cooldown_timer = weapon.stats.burst_cooldown_reset;
                                        weapon.burst_shots += 1;
                                    } else {
                                        weapon.cooldown_timer = weapon.stats.cooldown_reset;
                                        weapon.burst_shots = 0;
                                    }
                                }
                            }
                        }
                        weapon.cooldown_timer = (weapon.cooldown_timer - dt).max(-1.0);

                        let cooldown_pct = weapon.cooldown_timer / weapon.stats.cooldown_reset;
                        let tint_component = tints.get_mut(weapon.icon_entity);

                        if let Some(tint) = tint_component {
                            if cooldown_pct < 0.0 {
                                *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));
                            } else {
                                *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 0.15));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn calc_tracking_fire_angle(
    dist_to_selected_vehicle: f32,
    angle_to_selected_vehicle: f32,
    standard_angle: f32,
    weapon_tracking_angle: f32,
) -> f32 {
    let fire_angle;

    if dist_to_selected_vehicle <= 200.0 {
        if weapon_tracking_angle <= 0.001 {
            fire_angle = standard_angle;
        } else if weapon_tracking_angle >= PI {
            fire_angle = angle_to_selected_vehicle;
        } else {
            let mut angle_diff = standard_angle - angle_to_selected_vehicle;

            if angle_diff > PI {
                angle_diff = -(2.0 * PI - angle_diff);
            } else if angle_diff < -PI {
                angle_diff = -(-2.0 * PI - angle_diff);
            }

            if angle_diff.abs() < weapon_tracking_angle {
                fire_angle = angle_to_selected_vehicle;
            } else {
                fire_angle = standard_angle - weapon_tracking_angle * angle_diff / angle_diff.abs();
            }
        }
    } else {
        fire_angle = standard_angle; //no tracking, distance too far
    }

    fire_angle
}
