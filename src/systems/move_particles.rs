use amethyst::core::{Time, Transform};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Entities, Join, Read, System, SystemData, WriteStorage};

use crate::components::{Particles};

#[derive(SystemDesc)]
pub struct MoveParticlesSystem;

impl<'s> System<'s> for MoveParticlesSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Particles>,
        Read<'s, Time>,
    );

    fn run(
        &mut self,
        (entities, mut transforms, mut particles, time): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        for (entity, particle, transform) in (&entities, &mut particles, &mut transforms).join() {
            particle.life_timer -= dt;

            if particle.life_timer < 0.0 {
                let _ = entities.delete(entity);
            }
            else {
                transform.prepend_translation_x(particle.dx * dt);
                transform.prepend_translation_y(particle.dy * dt);
            }
        }
    }
}
