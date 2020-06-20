use amethyst::{
    core::{
        transform::{Transform, TransformBundle},
        Time,
    },
    derive::SystemDesc,
    ecs::{System, SystemData, Write, ReadStorage},
    renderer::{
        debug_drawing::{DebugLines},
        palette::Srgba,
    },
};

use crate::components::{Vehicle};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};


#[derive(SystemDesc)]
pub struct DebugLinesSystem;

impl<'s> System<'s> for DebugLinesSystem {
    type SystemData = (
        Write<'s, DebugLines>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Vehicle>,
    );

    fn run(&mut self, (mut debug_lines_resource, transforms, vehicles): Self::SystemData) {
        debug_lines_resource.draw_line(
            [0.0, UI_HEIGHT, 0.5].into(),
            [ARENA_WIDTH, ARENA_HEIGHT, 0.5].into(),
            Srgba::new(0.3, 0.3, 1.0, 1.0),
        );
    }
}