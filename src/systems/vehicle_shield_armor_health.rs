use amethyst::{
    core::{Time, Transform},
    derive::SystemDesc,
    ecs::{Join, Read, System, SystemData, WriteStorage, ReadExpect},
    input::{InputHandler, StringBindings},
    renderer::{palette::Srgba, resources::Tint},
};

use rand::Rng;
use std::collections::HashMap;

use crate::components::{
    Player, Vehicle, VehicleState, BotMode, vehicle_damage_model, kill_restart_vehicle, DurationDamage
};
use crate::resources::{GameModeSetup};


#[derive(SystemDesc)]
pub struct VehicleShieldArmorHealthSystem;

impl<'s> System<'s> for VehicleShieldArmorHealthSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Transform>,
        ReadExpect<'s, GameModeSetup>,
        WriteStorage<'s, Tint>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(
        &mut self,
        (mut players, mut vehicles, mut transforms, game_mode_setup, mut tints, time, input): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        let mut owner_data_map = HashMap::new();

        for (player, vehicle, vehicle_transform) in (&mut players, &mut vehicles, &transforms).join() {
            //Apply duration damage, such as poison/burns
            let mut vehicle_destroyed = false;

            let mut duration_damage_list = vehicle.duration_damage.clone();

            for duration_damage in duration_damage_list.iter() {
                if duration_damage.timer > 0.0 {
                    let duration_damage_vehicle_destroyed: bool = vehicle_damage_model(
                        vehicle,
                        duration_damage.damage_per_second.clone() * dt,
                        duration_damage.piercing_damage_pct.clone(),
                        duration_damage.shield_damage_pct.clone(),
                        duration_damage.armor_damage_pct.clone(),
                        duration_damage.health_damage_pct.clone(),
                        DurationDamage::default(), //These are zero/default so as to not re-apply the effect to the vehicle. 
                        //Otherwise this duration damage effect would stack continuously.
                    );

                    if duration_damage_vehicle_destroyed {
                        vehicle_destroyed = true;
                    }
                }
            }

            //Remove duration damages that have expired
            let mut lasting_duration_damages = Vec::<DurationDamage>::new();

            for duration_damage in duration_damage_list.iter_mut() {
                duration_damage.timer -= dt;

                if duration_damage.timer > 0.0 {
                    lasting_duration_damages.push(*duration_damage);
                }
            }

            vehicle.duration_damage = lasting_duration_damages;

            

            if vehicle_destroyed {
                player.deaths += 1;

                kill_restart_vehicle(
                    player,
                    vehicle,
                    vehicle_transform,
                    game_mode_setup.stock_lives,
                );
            }



            //Healing is automatically done if health is damaged
            if (vehicle.heal_pulse_rate > 0.0 && vehicle.health.value > 0.0) && 
                    (vehicle.health.max > 0.0 && vehicle.health.value < vehicle.health.max) {
                if vehicle.heal_cooldown_timer < 0.0 {
                    //healing applied
                    vehicle.health.value += vehicle.heal_pulse_amount;

                    vehicle.health.value = vehicle.health.value.min(vehicle.health.max);

                    vehicle.heal_cooldown_timer = vehicle.heal_pulse_rate;
                } else {
                    //waiting for heal pulse...
                    vehicle.heal_cooldown_timer -= dt;
                }
            }

            //Shields are automatically re-charged if shields are damaged
            if (vehicle.shield.value > 0.0) && 
                    (vehicle.shield.max > 0.0 && vehicle.shield.value < vehicle.shield.max) {
                if vehicle.shield.cooldown_timer < 0.0 {
                    //recharging
                    vehicle.shield.value += vehicle.shield.recharge_rate * dt;

                    vehicle.shield.value = vehicle.shield.value.min(vehicle.shield.max);

                    vehicle.shield.cooldown_timer = -1.0;
                } else {
                    //waiting for recharge...
                    //note that the cooldown timer is reset every time that the vehicle's shields are hit
                    vehicle.shield.cooldown_timer -= dt;
                }
            }


            //Repairing must be initiated by the player
            let vehicle_repair;
            if player.is_bot && player.bot_mode == BotMode::Repairing {
                vehicle_repair = Some(true);
            }
            else {
                vehicle_repair = match player.id {
                    0 => input.action_is_down("p1_repair"),
                    1 => input.action_is_down("p2_repair"),
                    2 => input.action_is_down("p3_repair"),
                    3 => input.action_is_down("p4_repair"),
                    _ => None,
                };
            }

            if let Some(repair) = vehicle_repair {
                if repair && vehicle.state == VehicleState::Active {
                    if vehicle.health.value < vehicle.health.max || (vehicle.shield.max > 0.0 && vehicle.shield.value == 0.0) {
                        //repair initiated
                        vehicle.repair.activated = true;
                        vehicle.repair.init_timer += dt;
                    } else {
                        //cancel
                        vehicle.repair.activated = false;
                        vehicle.repair.init_timer = 0.0;
                        vehicle.shield.repair_timer = 0.0;
                    }

                    if vehicle.repair.init_timer >= vehicle.repair.init_threshold {
                        //repair successful started
                        if vehicle.health.value < vehicle.health.max {
                            vehicle.health.value += vehicle.health.repair_rate * dt;
                            vehicle.health.value = vehicle.health.value.min(vehicle.health.max);
                        } else if vehicle.shield.value <= 0.0 && vehicle.shield.max > 0.0 {
                            vehicle.shield.repair_timer += dt;
                            if vehicle.shield.repair_timer > vehicle.shield.repair_threshold {
                                vehicle.shield.value = 1.0;
                            }
                        } else {
                            //completed
                            vehicle.repair.activated = false;
                            vehicle.repair.init_timer = 0.0;
                            vehicle.shield.repair_timer = 0.0;
                        }
                    }
                } else {
                    //cancel
                    vehicle.repair.activated = false;
                    vehicle.repair.init_timer = 0.0;
                    vehicle.shield.repair_timer = 0.0;
                }
            } else {
                //cancel
                vehicle.repair.activated = false;
                vehicle.repair.init_timer = 0.0;
                vehicle.shield.repair_timer = 0.0;
            }

            
            let vehicle_rotation = vehicle_transform.rotation();
            let (_, _, yaw) = vehicle_rotation.euler_angles();

            let vehicle_x = vehicle_transform.translation().x;
            let vehicle_y = vehicle_transform.translation().y;


            owner_data_map.insert(
                player.id,
                (
                    vehicle_x,
                    vehicle_y,
                    yaw,
                    vehicle.shield.value / vehicle.shield.max,
                    vehicle.armor.value / vehicle.armor.max,
                    vehicle.health.value / vehicle.health.max,
                    vehicle.repair.init_timer / vehicle.repair.init_threshold,
                    vehicle.shield.repair_timer / vehicle.shield.repair_threshold,
                ),
            );
        }

        //visual updates
        for (player, vehicle) in (&players, &mut vehicles).join() {
            let owner_data = owner_data_map.get(&player.id);

            if let Some(owner_data) = owner_data {
                let (
                    x,
                    y,
                    angle,
                    shield_pct,
                    armor_pct,
                    health_pct,
                    health_repair_pct,
                    shield_repair_pct,
                ) = owner_data;

                //Shield update
                {
                    let transform = transforms.get_mut(vehicle.shield.entity).unwrap();

                    transform.set_translation_x(*x);
                    transform.set_translation_y(*y);
                    transform.set_rotation_2d(*angle);

                    let tint = tints.get_mut(vehicle.shield.entity).unwrap();
                    if *shield_pct < 0.5 {
                        *tint = Tint(Srgba::new(1.0, 1.0, 1.0, (*shield_pct) * 2.0));
                    } else {
                        *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));
                    }
                }

                //Armor update
                {
                    let transform = transforms.get_mut(vehicle.armor.entity).unwrap();

                    transform.set_translation_x(*x);
                    transform.set_translation_y(*y);
                    transform.set_rotation_2d(*angle);

                    let tint = tints.get_mut(vehicle.armor.entity).unwrap();
                    if *armor_pct < 0.5 {
                        *tint = Tint(Srgba::new(1.0, 1.0, 1.0, *(armor_pct) * 2.0));
                    } else {
                        *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));
                    }
                }

                //Health update
                {
                    let transform = transforms.get_mut(vehicle.health.entity).unwrap();

                    transform.set_translation_x(*x);
                    transform.set_translation_y(*y);
                    transform.set_rotation_2d(*angle);

                    let tint = tints.get_mut(vehicle.health.entity).unwrap();
                    if *health_pct <= 0.0 {
                        *tint = Tint(Srgba::new(0.0, 0.0, 0.0, 1.0));
                    } else if *health_pct < (4. / 5.) {
                        *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0 - ((*health_pct) * (5. / 4.))));
                    } else {
                        *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 0.0));
                    }
                }

                //Repair update
                {
                    let transform = transforms.get_mut(vehicle.repair.entity).unwrap();

                    transform.set_translation_x(*x);
                    transform.set_translation_y(*y);
                    transform.set_rotation_2d(*angle);

                    let mut rng = rand::thread_rng();

                    let tint = tints.get_mut(vehicle.repair.entity).unwrap();
                    if *shield_repair_pct > 0.01 {
                        let blue = rng.gen_range(0.5, 1.0);
                        *tint = Tint(Srgba::new(0.0, 0.0, blue, blue));
                    } else if *health_repair_pct > 0.01 {
                        let red = rng.gen_range(0.5, 1.0);
                        *tint = Tint(Srgba::new(red, 0.0, 0.0, red));
                    } else {
                        *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 0.0));
                    }
                }
            }
        }
    }
}
