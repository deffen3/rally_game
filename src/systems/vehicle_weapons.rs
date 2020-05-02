use amethyst::{
    core::{math::Vector3, Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, LazyUpdate, Read, ReadExpect, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::{
        palette::Srgba,
        resources::Tint,
    },
};

use log::debug;
use rand::Rng;
use std::f32::consts::PI;

use crate::components::{BotMode, Player, Vehicle, Weapon};
use crate::rally::fire_weapon;
use crate::resources::WeaponFireResource;

#[derive(SystemDesc)]
pub struct VehicleWeaponsSystem;

impl<'s> System<'s> for VehicleWeaponsSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Weapon>,
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
            mut weapons,
            mut tints,
            weapon_fire_resource,
            lazy_update,
            time,
            input,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        let dt = time.delta_seconds();

        for (player, vehicle, weapon, transform) in
            (&mut players, &mut vehicles, &mut weapons, &mut transforms).join()
        {
            //let vehicle_weapon_fire = input.action_is_down(&ActionBinding::VehicleShoot(player.id));

            let mut vehicle_weapon_fire = match player.id {
                0 => input.action_is_down("p1_shoot"),
                1 => input.action_is_down("p2_shoot"),
                2 => input.action_is_down("p3_shoot"),
                3 => input.action_is_down("p4_shoot"),
                _ => None,
            };

            if player.is_bot {
                if player.bot_mode == BotMode::StopAim
                    || player.bot_mode == BotMode::Mining
                    || player.bot_mode == BotMode::Chasing
                    || player.bot_mode == BotMode::Swording
                {
                    vehicle_weapon_fire = Some(rng.gen::<bool>());
                }
            }

            if let Some(fire) = vehicle_weapon_fire {
                if vehicle.repair.activated == false {
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

                        let mut fire_angle;
                        if vehicle.dist_to_closest_vehicle <= 200.0 {
                            if weapon.stats.tracking_angle <= 0.001 {
                                fire_angle = vehicle_angle;
                            } else if weapon.stats.tracking_angle >= PI {
                                fire_angle = vehicle.angle_to_closest_vehicle;
                            } else {

                                let mut angle_diff = vehicle_angle - vehicle.angle_to_closest_vehicle;

                                if angle_diff > PI {
                                    angle_diff = -(2.0*PI - angle_diff);
                                }
                                else if angle_diff < -PI {
                                    angle_diff = -(-2.0*PI - angle_diff);
                                }
                                

                                if angle_diff.abs() < weapon.stats.tracking_angle {
                                    fire_angle = vehicle.angle_to_closest_vehicle;
                                }
                                else {
                                    fire_angle = vehicle_angle - weapon.stats.tracking_angle * angle_diff/angle_diff.abs();
                                }

                                if player.id == 0 {
                                    debug!("{}, {}",vehicle_angle, vehicle.angle_to_closest_vehicle);
                                }
                            }
                        }
                        else {
                            fire_angle = vehicle_angle; //no tracking, distance too far
                        }

                        if weapon.stats.spread_angle >= 0.001 {
                            let spread_angle_modifier = rng.gen_range(-1.0, 1.0) * weapon.stats.spread_angle;
                            fire_angle += spread_angle_modifier;
                        }

                        if !weapon.stats.attached
                            || (weapon.stats.attached && !weapon.stats.deployed)
                        {
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

                        if weapon.stats.burst_shot_limit > 0 && weapon.burst_shots < weapon.stats.burst_shot_limit {
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
                }
                else {
                    *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 0.15));
                }
            }
        }
    }
}
