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
pub struct VehicleShieldsSystem;

impl<'s> System<'s> for VehicleShieldsSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Tint>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (players, mut vehicles, mut transforms, mut tint, time, _input): Self::SystemData) {
        let dt = time.delta_seconds();

        let mut owner_data: Vec<(usize, f32, f32, f32, f32)> = Vec::new();

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

            owner_data.push((player.id,
                vehicle_x,
                vehicle_y,
                yaw, 
                vehicle.shield.value / vehicle.shield.max
            ));
        }

        for (player, vehicle) in (&players, &mut vehicles).join() {
            for (player_id_check, x, y, angle, damage_pct) in &owner_data {
                if *player_id_check == player.id {
                    let shield_transform = transforms.get_mut(vehicle.shield.entity).unwrap();
                    
                    shield_transform.set_translation_x(*x);
                    shield_transform.set_translation_y(*y);
                    shield_transform.set_rotation_2d(*angle);


                    if *damage_pct < 0.5 {
                        let shield_tint = tint.get_mut(vehicle.shield.entity).unwrap();
                        *shield_tint = Tint(Srgba::new(1.0, 1.0, 1.0, *damage_pct*2.0));
                    }
                }
            }
        }
    }
}
