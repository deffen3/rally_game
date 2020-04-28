use amethyst::core::{Time, Transform};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage, ReadStorage};
use amethyst::input::{InputHandler, StringBindings};

use crate::components::{Vehicle, Player};

#[derive(SystemDesc)]
pub struct VehicleShieldsSystem;

impl<'s> System<'s> for VehicleShieldsSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (players, mut vehicles, mut transforms, time, _input): Self::SystemData) {
        let dt = time.delta_seconds();

        let mut owner: Vec<(usize, f32, f32, f32)> = Vec::new();

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

            owner.push((player.id, vehicle_x, vehicle_y, yaw));
        }

        for (player, vehicle) in (&players, &mut vehicles).join() {
            for (player_id_check, x, y, angle) in &owner {
                if *player_id_check == player.id {
                    let shield_transform = transforms.get_mut(vehicle.shield.entity).unwrap();

                    shield_transform.set_translation_x(*x);
                    shield_transform.set_translation_y(*y);
                    shield_transform.set_rotation_2d(*angle);
                }
            }
        }
    }
}
