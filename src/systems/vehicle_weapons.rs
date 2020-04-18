use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage};
use amethyst::input::{InputHandler};


use crate::rally::{Vehicle, Player, ActionBinding, MovementBindingTypes, Weapon};

#[derive(SystemDesc)]
pub struct VehicleWeaponsSystem;

impl<'s> System<'s> for VehicleWeaponsSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Weapon>,
        Read<'s, Time>,
        Read<'s, InputHandler<MovementBindingTypes>>,
    );

    fn run(&mut self, (mut players, mut transforms, mut vehicles, mut weapons, time, input): Self::SystemData) {
        let dt = time.delta_seconds();

        for (player, vehicle, weapon, transform) in (&mut players, &mut vehicles, &mut weapons, &mut transforms).join() {
            let vehicle_weapon_fire = input.action_is_down(&ActionBinding::VehicleShoot(player.id));

            if let Some(fire) = vehicle_weapon_fire {
                if fire && weapon.weapon_cooldown_timer <= 0.0 {
                    println!("Fire {}", player.id);
                    weapon.weapon_cooldown_timer = 2.0;
                }
            }
            weapon.weapon_cooldown_timer = (weapon.weapon_cooldown_timer - dt).max(-1.0);
        }
    }
}