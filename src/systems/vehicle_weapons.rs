use amethyst::{
    core::{math::Vector3, Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, LazyUpdate, Read, ReadExpect, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::{palette::Srgba, resources::Tint},
};

use rand::Rng;
use std::f32::consts::PI;

use crate::components::{BotMode, Player, Vehicle, VehicleState, WeaponArray};
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
                //let vehicle_weapon_is_firing = input.action_is_down(&ActionBinding::VehicleShoot(player.id));
                
                let primary_fire: Option<bool>;
                let secondary_fire: Option<bool>;

                if player.is_bot {
                    if player.bot_mode == BotMode::StopAim
                        || player.bot_mode == BotMode::StrafeAim
                        || player.bot_mode == BotMode::Mining
                        || player.bot_mode == BotMode::Chasing
                        || player.bot_mode == BotMode::Swording
                        || player.bot_mode == BotMode::Racing
                    {
                        primary_fire = Some(true);
                        secondary_fire = Some(rng.gen::<bool>());
                    }
                    else {
                        primary_fire = None;
                        secondary_fire = None;
                    }
                }
                else {
                    primary_fire = match player.id {
                        0 => input.action_is_down("p1_fire"),
                        1 => input.action_is_down("p2_fire"),
                        2 => input.action_is_down("p3_fire"),
                        3 => input.action_is_down("p4_fire"),
                        _ => None,
                    };
    
                    secondary_fire = match player.id {
                        0 => input.action_is_down("p1_alt_fire"),
                        1 => input.action_is_down("p2_alt_fire"),
                        2 => input.action_is_down("p3_alt_fire"),
                        3 => input.action_is_down("p4_alt_fire"),
                        _ => None,
                    };
                }


                for (weapon_index, weapon_install) in weapon_array.installed.iter_mut().enumerate() {
                    let weapon_is_attempting_to_fire: bool;
                    
                    if (weapon_install.firing_group == 0 && !primary_fire.is_none() && primary_fire.unwrap())
                        || (weapon_install.firing_group == 1 && !secondary_fire.is_none() && secondary_fire.unwrap()) 
                    {
                        weapon_is_attempting_to_fire = true;
                    }
                    else {
                        weapon_is_attempting_to_fire = false;
                    }

                        
                    let weapon = &mut weapon_install.weapon;

                    if weapon.deploy_timer > 0.0 {
                        weapon.deploy_timer -= dt;
                    }

                    //if trying to fire weapon
                    //Manage cooldown, burst, spin-up, and charging delays
                    if weapon_is_attempting_to_fire && !vehicle.repair.activated {
                        if weapon.cooldown_timer <= 0.0 {
                            if weapon.spin_up_timer <= 0.0 {
                                if weapon.charge_timer <= 0.0 {
                                    //if weapon.ammo is None, this means infinite ammo
                                    if weapon.ammo.is_none() || weapon.ammo.unwrap() > 0 {
                                        //finally, the weapon can now fire
                                        if !weapon.ammo.is_none() {
                                            weapon.ammo = Some(weapon.ammo.unwrap() - 1);
                                        }

                                        let vehicle_rotation = transform.rotation();
                                        let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();

                                        let yaw_x_comp = -vehicle_angle.sin(); //left is -, right is +
                                        let yaw_y_comp = vehicle_angle.cos(); //up is +, down is -

                                        let fire_position = Vector3::new(
                                            transform.translation().x + yaw_x_comp * 5.0,
                                            transform.translation().y + yaw_y_comp * 5.0,
                                            0.0,
                                        );

                                        let install_mounted_angle;
                                        if weapon_install.mounted_angle.is_none() {
                                            install_mounted_angle = 0.0;
                                        }
                                        else {
                                            install_mounted_angle = weapon_install.mounted_angle.unwrap();
                                        }

                                        //typical angle this weapon should fire at
                                        let standard_angle = vehicle_angle + install_mounted_angle + 
                                            weapon.stats.fire_stats.mount_angle_special_offset;
                                    

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




                                        //if attached weapon is already deployed, then undeploy it
                                        if weapon.stats.fire_stats.attached && weapon.deployed {
                                            if weapon.deploy_timer <= 0.0 {
                                                weapon.deploy_timer = 1.0; //reset cooldown
                                                weapon.deployed = false;
                                            }
                                        }
                                        else if !weapon.stats.fire_stats.attached || !weapon.deployed {
                                            if !weapon.deployed {
                                                if weapon.deploy_timer <= 0.0 {
                                                    weapon.deploy_timer = 1.0; //reset cooldown
                                                    weapon.deployed = true;
                                                }
                                            }
                                        }

                                        if weapon.deployed {
                                            fire_weapon(
                                                &entities,
                                                &weapon_fire_resource,
                                                weapon.clone(),
                                                weapon_index,
                                                fire_position,
                                                fire_angle,
                                                Some(player.id),
                                                &lazy_update,
                                            );
                                        }

                                        
                                        //manage cooldown timer reset and burst fire reset
                                        if weapon.stats.burst_shot_limit > 0
                                            && weapon.burst_shots < weapon.stats.burst_shot_limit
                                        {
                                            weapon.cooldown_timer = weapon.stats.burst_cooldown_reset;
                                            weapon.burst_shots += 1;
                                        } else {
                                            weapon.cooldown_timer = weapon.stats.cooldown_reset;
                                            weapon.burst_shots = 0;
                                        }

                                        weapon.charges += 1;
                                        weapon.charge_timer = weapon.stats.charge_timer_reset 
                                            - (weapon.charges as f32)*weapon.stats.charge_timer_decrease;
                                            
                                        if weapon.charge_timer < weapon.stats.charge_timer_decrease_min {
                                            weapon.charge_timer = weapon.stats.charge_timer_decrease_min;
                                        }
                                    }
                                    else {
                                        //out of ammo, some type UI interaction here?
                                    }
                                }
                                else { //waiting for this shot to charge
                                    weapon.charge_timer -= dt;
                                }
                            }
                            else { //waiting for weapon to spin up
                                weapon.spin_up_timer -= dt;
                            }
                        }
                    }
                    else { //stopped firing, reset spin-up timer
                        weapon.spin_up_timer = weapon.stats.spin_up_timer_reset;

                        weapon.charge_timer = weapon.stats.charge_timer_reset;
                        weapon.charges = 0;
                    }


                    //manage cooldown timer and weapon icon indicators
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
