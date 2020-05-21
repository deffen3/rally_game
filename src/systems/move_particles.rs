use amethyst::core::{Time, Transform};
use amethyst::core::math::Vector3;
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Entities, Join, Read, System, SystemData, WriteStorage, ReadStorage};

use crate::components::{Particles, Shockwave};

#[derive(SystemDesc)]
pub struct MoveParticlesSystem;

impl<'s> System<'s> for MoveParticlesSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Particles>,
        ReadStorage<'s, Shockwave>,
        Read<'s, Time>,
    );

    fn run(
        &mut self,
        (entities, mut transforms, mut particles, shockwaves, time): Self::SystemData,
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

        for (particle, shockwave, transform) in (&mut particles, &shockwaves, &mut transforms).join() {
            let pct_expansion = 1.0 - (particle.life_timer / shockwave.time);
            let live_shockwave_radius = pct_expansion * shockwave.radius/2.0;

            //sprite is 21x21 pixels, we'll call is radius=10
            //so scale of 1 = 10 pixel radius
            let scale = live_shockwave_radius / 10.0;

            transform.set_scale(Vector3::new(scale, scale, 0.0));
        }
    }
}
