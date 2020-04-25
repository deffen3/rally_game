use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, ReadExpect, System, SystemData, WriteStorage, Entities, LazyUpdate};
use amethyst::input::{InputHandler, StringBindings};
use amethyst::core::math::Vector3;


use crate::components::{Vehicle, Player, Weapon};
use crate::resources::{WeaponFireResource};
use crate::rally::{fire_weapon};

#[derive(SystemDesc)]
pub struct VehicleWeaponsSystem;


impl<'s> System<'s> for VehicleWeaponsSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Weapon>,
        ReadExpect<'s, WeaponFireResource>,
        ReadExpect<'s, LazyUpdate>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>, //<MovementBindingTypes>>,
    );

    fn run(&mut self, (entities, mut players, mut transforms, 
            mut vehicles, mut weapons, weapon_fire_resource, lazy_update, time, input):
            Self::SystemData) {

        let dt = time.delta_seconds();

        for (player, _vehicle, weapon, transform) in (&mut players, &mut vehicles, &mut weapons, &mut transforms).join() {
            //let vehicle_weapon_fire = input.action_is_down(&ActionBinding::VehicleShoot(player.id));

            let vehicle_weapon_fire = match player.id {
                0 => input.action_is_down("p1_shoot"),
                1 => input.action_is_down("p2_shoot"),
                2 => input.action_is_down("p3_shoot"),
                3 => input.action_is_down("p4_shoot"),
                _ => None
            };


            if let Some(fire) = vehicle_weapon_fire {
                if fire && weapon.weapon_cooldown_timer <= 0.0 {

                    let vehicle_rotation = transform.rotation();
                    let (_, _, yaw) = vehicle_rotation.euler_angles();

                    let yaw_x_comp = -yaw.sin(); //left is -, right is +
                    let yaw_y_comp = yaw.cos(); //up is +, down is -

                    let fire_position = Vector3::new(
                        transform.translation().x + yaw_x_comp*5.0,
                        transform.translation().y + yaw_y_comp*5.0,
                        0.0,
                    );

                    let vehicle_rotation = transform.rotation();
                    let (_, _, fire_angle) = vehicle_rotation.euler_angles();

                    if weapon.attached == false || (weapon.attached == true && weapon.deployed == false) {
                        if weapon.deployed == false {
                            weapon.deployed = true;
                        }
                        fire_weapon(&entities, &weapon_fire_resource, weapon.clone(),
                            fire_position, fire_angle, player.id, &lazy_update);
                    }

                    if fire && weapon.burst_shots < weapon.burst_shot_limit {
                        weapon.weapon_cooldown_timer = weapon.burst_cooldown_reset;
                        weapon.burst_shots += 1;
                    }
                    else {
                        weapon.weapon_cooldown_timer = weapon.weapon_cooldown_reset;
                        weapon.burst_shots = 0;
                    }
                    
                }
            }
            weapon.weapon_cooldown_timer = (weapon.weapon_cooldown_timer - dt).max(-1.0);
        }
    }
}