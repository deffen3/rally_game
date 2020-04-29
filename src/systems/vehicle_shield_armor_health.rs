use amethyst::{
    core::{Time, Transform},
    derive::SystemDesc,
    ecs::{Join, Read, System, SystemData, WriteStorage, ReadStorage},
    input::{InputHandler, StringBindings},
    renderer::{
        palette::Srgba,
        resources::Tint,
    },
};

use crate::components::{Vehicle, Player};

#[derive(SystemDesc)]
pub struct VehicleShieldArmorHealthSystem;

impl<'s> System<'s> for VehicleShieldArmorHealthSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Tint>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (players, mut vehicles, mut transforms, mut tints, time, input): Self::SystemData) {
        let dt = time.delta_seconds();

        let mut owner_data: Vec<(usize, f32, f32, f32, f32, f32, f32, f32, f32)> = Vec::new();

        for (player, vehicle, vehicle_transform) in (&players, &mut vehicles, &transforms).join() {
            if (vehicle.shield.value > 0.0) && (vehicle.shield.value < vehicle.shield.max) {
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

            let vehicle_rotation = vehicle_transform.rotation();
            let (_, _, yaw) = vehicle_rotation.euler_angles();

            let vehicle_x = vehicle_transform.translation().x;
            let vehicle_y = vehicle_transform.translation().y;


            let vehicle_repair = match player.id {
                0 => input.action_is_down("p1_repair"),
                1 => input.action_is_down("p2_repair"),
                2 => input.action_is_down("p3_repair"),
                3 => input.action_is_down("p4_repair"),
                _ => None,
            };

            if let Some(repair) = vehicle_repair {
                if repair {
                    if vehicle.health.value < vehicle.health.max {
                        //repair initiated
                        vehicle.repair.activated = true;
                        vehicle.repair.cooldown_timer += dt;
                    } else if vehicle.shield.value == 0.0 {
                        //repair initiated
                        vehicle.repair.activated = true;
                        vehicle.repair.cooldown_timer += dt;
                    } else { //cancel
                        vehicle.repair.activated = false;
                        vehicle.repair.cooldown_timer = 0.0;
                        vehicle.shield.repair_timer = 0.0;
                    }

                    if vehicle.repair.cooldown_timer >= vehicle.repair.cooldown_threshold {
                        //repair successful started
                        if vehicle.health.value < vehicle.health.max {
                            vehicle.health.value += vehicle.health.repair_rate * dt;
                            vehicle.health.value = vehicle.health.value.min(100.0);
                        } else if vehicle.shield.value <= 0.0 && vehicle.shield.max > 0.0 {
                            vehicle.shield.repair_timer += dt;
                            if vehicle.shield.repair_timer > vehicle.shield.repair_threshold {
                                vehicle.shield.value = 1.0;
                            }
                        } else { //completed
                            vehicle.repair.activated = false;
                            vehicle.repair.cooldown_timer = 0.0;
                            vehicle.shield.repair_timer = 0.0;
                        }
                    }
                }
                else { //cancel
                    vehicle.repair.activated = false;
                    vehicle.repair.cooldown_timer = 0.0;
                    vehicle.shield.repair_timer = 0.0;
                }
            }
            else { //cancel
                vehicle.repair.activated = false;
                vehicle.repair.cooldown_timer = 0.0;
                vehicle.shield.repair_timer = 0.0;
            }

            owner_data.push((player.id,
                vehicle_x,
                vehicle_y,
                yaw, 
                vehicle.shield.value / vehicle.shield.max,
                vehicle.armor.value / vehicle.armor.max,
                vehicle.health.value / vehicle.health.max,
                vehicle.repair.cooldown_timer / vehicle.repair.cooldown_threshold,
                vehicle.shield.repair_timer / vehicle.shield.repair_threshold,
            ));
        }



        //visual updates
        for (player, vehicle) in (&players, &mut vehicles).join() {
            for (player_id_check, x, y, angle, 
                    shield_pct, armor_pct, health_pct,
                    health_repair_pct, shield_repair_pct,
            ) in &owner_data {
                if *player_id_check == player.id {
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
                        if *health_pct < (4./5.) {
                            *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0-((*health_pct) * (5./4.))));
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

                        let tint = tints.get_mut(vehicle.repair.entity).unwrap();
                        if *health_repair_pct > 0.01 {
                            *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));
                        } else if *shield_repair_pct > 0.01 {
                            *tint = Tint(Srgba::new(0.0, 0.0, 1.0, 1.0));
                        } else {
                            *tint = Tint(Srgba::new(1.0, 1.0, 1.0, 0.0));
                        }
                    }
                }
            }
        }
    }
}
