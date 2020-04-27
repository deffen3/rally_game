use amethyst::core::Time;
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage};
use amethyst::input::{InputHandler, StringBindings};

use crate::components::Vehicle;

#[derive(SystemDesc)]
pub struct VehicleShieldsSystem;

impl<'s> System<'s> for VehicleShieldsSystem {
    type SystemData = (
        WriteStorage<'s, Vehicle>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (mut vehicles, time, _input): Self::SystemData) {
        let dt = time.delta_seconds();

        for vehicle in (&mut vehicles).join() {
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
        }
    }
}
