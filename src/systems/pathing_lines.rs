use amethyst::{
    core::{
        transform::{Transform},
    },
    derive::SystemDesc,
    ecs::{System, Join, SystemData, Write, ReadExpect, WriteStorage, ReadStorage},
    renderer::{
        debug_drawing::{DebugLines},
        palette::Srgba,
    },
};


use navmesh::{NavQuery, NavPathMode};


use crate::components::{Vehicle, Player};
use crate::resources::{ArenaNavMesh, ArenaInvertedNavMesh, ArenaNavMeshFinal};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT, DEBUG_LINES};


#[derive(SystemDesc)]
pub struct PathingLinesSystem;

impl<'s> System<'s> for PathingLinesSystem {
    type SystemData = (
        Write<'s, DebugLines>,
        ReadExpect<'s, ArenaNavMesh>,
        ReadExpect<'s, ArenaInvertedNavMesh>,
        ReadExpect<'s, ArenaNavMeshFinal>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        ReadStorage<'s, Transform>,
    );

    fn run(
        &mut self, (
        mut debug_lines_resource, 
        arena_nav_mesh,
        _arena_inv_nav_mesh,
        arena_nav_mesh_final,
        mut players,
        vehicles,
        transforms): Self::SystemData
    ) {
        // debug_lines_resource.draw_line(
        //     [0.0, UI_HEIGHT, 0.5].into(),
        //     [ARENA_WIDTH, ARENA_HEIGHT, 0.5].into(),
        //     Srgba::new(0.3, 0.3, 1.0, 1.0),
        // );

        
        let nav_query_type = NavQuery::Accuracy;
        let nav_path_type = NavPathMode::Accuracy;
        /*
        /// Quality of querying a point on nav mesh.
        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        pub enum NavQuery {
            /// Best quality, totally accurate.
            Accuracy,
            /// Medium quality, finds point in closest triangle.
            Closest,
            /// Low quality, finds first triangle in range of query.
            ClosestFirst,
        }

        /// Quality of finding path.
        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        pub enum NavPathMode {
            /// Best quality, finds shortest path.
            Accuracy,
            /// Medium quality, finds shortest path througs triangles midpoints.
            MidPoints,
        }
        */


        //draw keep-out zone as red debug lines
        // for (v1_index, v2_index, v3_index) in arena_inv_nav_mesh.triangles.iter() {
        //     let v1 = arena_inv_nav_mesh.vertices[*v1_index];
        //     let v2 = arena_inv_nav_mesh.vertices[*v2_index];
        //     let v3 = arena_inv_nav_mesh.vertices[*v3_index];

        //     debug_lines_resource.draw_line(
        //         [v1.0, v1.1, v1.2].into(),
        //         [v2.0, v2.1, v2.2].into(),
        //         Srgba::new(1.0, 0.2, 0.2, 1.0),
        //     );

        //     debug_lines_resource.draw_line(
        //         [v2.0, v2.1, v2.2].into(),
        //         [v3.0, v3.1, v3.2].into(),
        //         Srgba::new(1.0, 0.2, 0.2, 1.0),
        //     );

        //     debug_lines_resource.draw_line(
        //         [v3.0, v3.1, v3.2].into(),
        //         [v1.0, v1.1, v1.2].into(),
        //         Srgba::new(1.0, 0.2, 0.2, 1.0),
        //     );
        // }

        //draw nav mesh zones as green debug lines
        if DEBUG_LINES {
            for (v1_index, v2_index, v3_index) in arena_nav_mesh.triangles.iter() {
                let v1 = arena_nav_mesh.vertices[*v1_index];
                let v2 = arena_nav_mesh.vertices[*v2_index];
                let v3 = arena_nav_mesh.vertices[*v3_index];

                debug_lines_resource.draw_line(
                    [v1.0, v1.1, v1.2].into(),
                    [v2.0, v2.1, v2.2].into(),
                    Srgba::new(0.2, 1.0, 0.2, 0.1),
                );

                debug_lines_resource.draw_line(
                    [v2.0, v2.1, v2.2].into(),
                    [v3.0, v3.1, v3.2].into(),
                    Srgba::new(0.2, 1.0, 0.2, 0.1),
                );

                debug_lines_resource.draw_line(
                    [v3.0, v3.1, v3.2].into(),
                    [v1.0, v1.1, v1.2].into(),
                    Srgba::new(0.2, 1.0, 0.2, 0.1),
                );
            }
        }
        

        for (player, _vehicle, transform) in (&mut players, &vehicles, &transforms).join()
        {
            let vehicle_x = transform.translation().x;
            let vehicle_y = transform.translation().y;

            if let Some(mesh) = &arena_nav_mesh_final.mesh {

                if !player.is_bot {
                    //player.path_target = Some((0.0, 0.0, 0.0));
                    player.path_target = Some((ARENA_WIDTH/2.0, (UI_HEIGHT + ARENA_HEIGHT)/2.0 , 0.5));
                }

                if let Some(target) = player.path_target {
                    let path = mesh
                        .find_path(
                            (vehicle_x, vehicle_y, 0.0).into(),
                            target.into(),
                            nav_query_type,
                            nav_path_type,
                        );
                    
                    if let Some(path) = path {
                        if DEBUG_LINES {
                            let mut prev_x = None;
                            let mut prev_y = None;
                            let mut prev_z = None;

                            let mut path_plan = Vec::new();

                            for nav_vector in path.iter() {
                                let x = nav_vector.x;
                                let y = nav_vector.y;
                                let z = nav_vector.z;

                                path_plan.push((x, y, z));

                                if !prev_x.is_none() {
                                    debug_lines_resource.draw_line(
                                        [prev_x.unwrap(), prev_y.unwrap(), prev_z.unwrap()].into(),
                                        [x, y, z].into(),
                                        Srgba::new(0.2, 0.8, 1.0, 0.3),
                                    );
                                }

                                prev_x = Some(x);
                                prev_y = Some(y);
                                prev_z = Some(z);
                            }

                            player.path_plan = Some(path_plan.clone());
                        }
                    }
                }
                else {
                    player.path_plan = None;
                }
            }
        }
    }
}