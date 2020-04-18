use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage};


use crate::rally::{WeaponFire};

#[derive(SystemDesc)]
pub struct MoveWeaponFireSystem;

impl<'s> System<'s> for MoveWeaponFireSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WeaponFire>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut weapon_fires, time): Self::SystemData) {
        let dt = time.delta_seconds();

        for (weapon_fire, transform) in (&mut weapon_fires, &mut transforms).join() {
            if weapon_fire.active == true {
                transform.set_rotation_2d(weapon_fire.spawn_angle);

                transform.prepend_translation_x(weapon_fire.spawn_x);
                transform.prepend_translation_y(weapon_fire.spawn_y);
            }
        }
    }
}