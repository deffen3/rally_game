use amethyst::{
    core::{
        transform::{Transform},
    },
    derive::SystemDesc,
    ecs::{System, SystemData, Write, ReadExpect, ReadStorage},
    renderer::{
        debug_drawing::{DebugLines},
        palette::Srgba,
    },
};

use crate::components::{Vehicle, Player};
use crate::resources::{ArenaNavMesh};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};


#[derive(SystemDesc)]
pub struct DebugLinesSystem;

impl<'s> System<'s> for DebugLinesSystem {
    type SystemData = (
        Write<'s, DebugLines>,
        ReadExpect<'s, ArenaNavMesh>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        ReadStorage<'s, Transform>,
    );

    fn run(&mut self, (mut debug_lines_resource, arena_nav_mesh, _players, _vehicles, _transforms): Self::SystemData) {
        debug_lines_resource.draw_line(
            [0.0, UI_HEIGHT, 0.5].into(),
            [ARENA_WIDTH, ARENA_HEIGHT, 0.5].into(),
            Srgba::new(0.3, 0.3, 1.0, 1.0),
        );

        for (v1_index, v2_index, v3_index) in arena_nav_mesh.triangles.iter() {
            let v1 = arena_nav_mesh.vertices[*v1_index];
            let v2 = arena_nav_mesh.vertices[*v2_index];
            let v3 = arena_nav_mesh.vertices[*v3_index];

            debug_lines_resource.draw_line(
                [v1.0, v1.1, v1.2].into(),
                [v2.0, v2.1, v2.2].into(),
                Srgba::new(1.0, 0.2, 0.2, 1.0),
            );

            debug_lines_resource.draw_line(
                [v2.0, v2.1, v2.2].into(),
                [v3.0, v3.1, v3.2].into(),
                Srgba::new(0.2, 0.1, 0.2, 1.0),
            );

            debug_lines_resource.draw_line(
                [v3.0, v3.1, v3.2].into(),
                [v1.0, v1.1, v1.2].into(),
                Srgba::new(0.2, 0.2, 1.0, 1.0),
            );
        }
    }
}