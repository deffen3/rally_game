use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage, Entities};


use crate::rally::{WeaponFire, ARENA_WIDTH, ARENA_HEIGHT};

#[derive(SystemDesc)]
pub struct MoveWeaponFireSystem;

impl<'s> System<'s> for MoveWeaponFireSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WeaponFire>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, mut transforms, mut weapon_fires, time): Self::SystemData) {
        let dt = time.delta_seconds();

        for (entity, weapon_fire, transform) in (&*entities, &mut weapon_fires, &mut transforms).join() {
            transform.prepend_translation_x(weapon_fire.dx * dt);
            transform.prepend_translation_y(weapon_fire.dy * dt);

            //out of arena logic
            let fire_x = transform.translation().x;
            let fire_y = transform.translation().y;

            if (fire_x > (ARENA_WIDTH + 2.0*weapon_fire.width)) || 
                    (fire_x < (-2.0*weapon_fire.width)) || 
                    (fire_y > (ARENA_HEIGHT + 2.0*weapon_fire.width)) ||
                    (fire_y < (-2.0*weapon_fire.width)) {

                let _ = entities.delete(entity);
            }
        }
    }
}