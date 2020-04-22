use amethyst::core::{Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage};
use amethyst::input::{InputHandler, StringBindings};

use crate::rally::{Vehicle};

#[derive(SystemDesc)]
pub struct VehicleShieldsSystem;


impl<'s> System<'s> for VehicleShieldsSystem {
    type SystemData = (
        WriteStorage<'s, Vehicle>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (mut vehicles, time, _input):
            Self::SystemData) {

        let dt = time.delta_seconds();

        for vehicle in (&mut vehicles).join() {
            /*
            vehicle.shield
            //shield_recharge_rate: 5.0,
            //shield_cooldown_timer: -1.0,
            //shield_cooldown_reset: 10.0,
            */

            if (vehicle.shield > 0.0) && (vehicle.shield < vehicle.shield_max) {
                if vehicle.shield_cooldown_timer < 0.0 {
                    //recharging
                    vehicle.shield += vehicle.shield_recharge_rate * dt;

                    vehicle.shield = vehicle.shield.min(vehicle.shield_max);

                    vehicle.shield_cooldown_timer = -1.0;
                }
                else {
                    //waiting for recharge...
                    //note that the cooldown timer is reset every time that the vehicle's shields are hit
                    vehicle.shield_cooldown_timer -= dt;
                }
            }
        }
    }
}