use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, ReadExpect, System, SystemData, WriteStorage, Entities, LazyUpdate};
use amethyst::input::{InputHandler};
use amethyst::core::math::Vector3;

use crate::rally::{Vehicle, Player, ActionBinding, MovementBindingTypes, 
    Weapon, WeaponFire, WeaponFireResource, fire_weapon, WEAPON_COOLDOWN};

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
        Read<'s, InputHandler<MovementBindingTypes>>,
    );

    fn run(&mut self, (entities, mut players, mut transforms, 
            mut vehicles, mut weapons, weapon_fire_resource, lazy_update, time, input):
            Self::SystemData) {

        let dt = time.delta_seconds();

        for (player, vehicle, weapon, transform) in (&mut players, &mut vehicles, &mut weapons, &mut transforms).join() {
            let vehicle_weapon_fire = input.action_is_down(&ActionBinding::VehicleShoot(player.id));

            if let Some(fire) = vehicle_weapon_fire {
                if fire && weapon.weapon_cooldown_timer <= 0.0 {

                    let fire_position = Vector3::new(
                        transform.translation().x,
                        transform.translation().y,
                        0.0,
                    );

                    let vehicle_rotation = transform.rotation();
                    let (_, _, fire_angle) = vehicle_rotation.euler_angles();

                    fire_weapon(&entities, &weapon_fire_resource, fire_position, fire_angle, &lazy_update);

                    weapon.weapon_cooldown_timer = WEAPON_COOLDOWN;
                }
            }
            weapon.weapon_cooldown_timer = (weapon.weapon_cooldown_timer - dt).max(-1.0);
        }
    }
}