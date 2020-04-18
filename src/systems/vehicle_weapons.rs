use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage};
use amethyst::input::{InputHandler};


use crate::rally::{Vehicle, ActionBinding, MovementBindingTypes, Weapon};

#[derive(SystemDesc)]
pub struct VehicleWeaponsSystem;

impl<'s> System<'s> for VehicleWeaponsSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Weapon>,
        Read<'s, Time>,
        Read<'s, InputHandler<MovementBindingTypes>>,
    );

    fn run(&mut self, (mut transforms, mut vehicles, mut weapons, time, input): Self::SystemData) {
        let dt = time.delta_seconds();

        // for (weapon, transform) in (&mut weapons, &mut transforms).join() {
        //     let vehicle_weapon_fire = input.action_is_down(&ActionBinding::VehicleShoot(vehicle.id));

        //     weapon.weapon_cooldown_timer = (weapon.weapon_cooldown_timer - dt).max(-1.0);
        // }
    }
}