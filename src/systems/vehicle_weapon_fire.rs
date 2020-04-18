use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage};
use amethyst::input::{InputHandler};


use crate::rally::{MovementBindingTypes, Weapon};

#[derive(SystemDesc)]
pub struct VehicleWeaponFireSystem;

impl<'s> System<'s> for VehicleWeaponFireSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Weapon>,
        Read<'s, Time>,
        Read<'s, InputHandler<MovementBindingTypes>>,
    );

    fn run(&mut self, (mut transforms, mut weapons, time, input): Self::SystemData) {
        let dt = time.delta_seconds();

        for (weapon, transform) in (&mut weapons, &mut transforms).join() {

        }
    }
}