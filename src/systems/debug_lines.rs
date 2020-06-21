use amethyst::{
    core::{
        transform::{Transform},
    },
    derive::SystemDesc,
    ecs::{System, Join, SystemData, Write, ReadExpect, ReadStorage},
    renderer::{
        debug_drawing::{DebugLines},
        palette::Srgba,
    },
};


use navmesh::{NavMesh, NavQuery, NavPathMode, NavVec3, NavTriangle};


use crate::components::{Vehicle, Player};
use crate::resources::{ArenaNavMesh, ArenaInvertedNavMesh};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};



#[derive(SystemDesc)]
pub struct DebugLinesSystem;

impl<'s> System<'s> for DebugLinesSystem {
    type SystemData = (
        Write<'s, DebugLines>,
        ReadExpect<'s, ArenaNavMesh>,
        ReadExpect<'s, ArenaInvertedNavMesh>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        ReadStorage<'s, Transform>,
    );

    fn run(
        &mut self, (
        mut debug_lines_resource, 
        arena_nav_mesh,
        arena_inv_nav_mesh,
        players,
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
        for (v1_index, v2_index, v3_index) in arena_inv_nav_mesh.triangles.iter() {
            let v1 = arena_inv_nav_mesh.vertices[*v1_index];
            let v2 = arena_inv_nav_mesh.vertices[*v2_index];
            let v3 = arena_inv_nav_mesh.vertices[*v3_index];

            debug_lines_resource.draw_line(
                [v1.0, v1.1, v1.2].into(),
                [v2.0, v2.1, v2.2].into(),
                Srgba::new(1.0, 0.2, 0.2, 1.0),
            );

            debug_lines_resource.draw_line(
                [v2.0, v2.1, v2.2].into(),
                [v3.0, v3.1, v3.2].into(),
                Srgba::new(1.0, 0.2, 0.2, 1.0),
            );

            debug_lines_resource.draw_line(
                [v3.0, v3.1, v3.2].into(),
                [v1.0, v1.1, v1.2].into(),
                Srgba::new(1.0, 0.2, 0.2, 1.0),
            );
        }

        //draw nav mesh zones as green debug lines
        for (v1_index, v2_index, v3_index) in arena_nav_mesh.triangles.iter() {
            let v1 = arena_nav_mesh.vertices[*v1_index];
            let v2 = arena_nav_mesh.vertices[*v2_index];
            let v3 = arena_nav_mesh.vertices[*v3_index];

            debug_lines_resource.draw_line(
                [v1.0, v1.1, v1.2].into(),
                [v2.0, v2.1, v2.2].into(),
                Srgba::new(0.2, 1.0, 0.2, 1.0),
            );

            debug_lines_resource.draw_line(
                [v2.0, v2.1, v2.2].into(),
                [v3.0, v3.1, v3.2].into(),
                Srgba::new(0.2, 1.0, 0.2, 1.0),
            );

            debug_lines_resource.draw_line(
                [v3.0, v3.1, v3.2].into(),
                [v1.0, v1.1, v1.2].into(),
                Srgba::new(0.2, 1.0, 0.2, 1.0),
            );
        }

        let mut final_arena_nav_mesh_vertices: Vec<NavVec3> = Vec::new();

        for (x,y,z) in arena_nav_mesh.vertices.iter() {
            final_arena_nav_mesh_vertices.push(NavVec3::new(*x, *y, *z));
        }


        let mut final_arena_nav_mesh_triangles: Vec<NavTriangle> = Vec::new();

        for (v1, v2, v3) in arena_nav_mesh.triangles.iter() {
            final_arena_nav_mesh_triangles.push(NavTriangle {
                first: *v1 as u32,
                second: *v2 as u32,
                third: *v3 as u32
            });
        }

        

        for (_player, _vehicle, transform) in (&players, &vehicles, &transforms).join()
        {
            let vehicle_x = transform.translation().x;
            let vehicle_y = transform.translation().y;

            let mesh = NavMesh::new(final_arena_nav_mesh_vertices.clone(), final_arena_nav_mesh_triangles.clone()).unwrap();

            let path = mesh
                .find_path(
                    (vehicle_x, vehicle_y, 0.5).into(),
                    (60.0, ARENA_HEIGHT + UI_HEIGHT - 50.0, 0.5).into(),
                    nav_query_type,
                    nav_path_type,
                );

            //draw best path as blue debug line
            if let Some(path) = path {
                let mut prev_x = None;
                let mut prev_y = None;
                let mut prev_z = None;

                for nav_vector in path.iter() {
                    let x = nav_vector.x;
                    let y = nav_vector.y;
                    let z = nav_vector.z;

                    if !prev_x.is_none() {
                        debug_lines_resource.draw_line(
                            [prev_x.unwrap(), prev_y.unwrap(), prev_z.unwrap()].into(),
                            [x, y, z].into(),
                            Srgba::new(0.2, 0.2, 1.0, 1.0),
                        );
                    }

                    prev_x = Some(x);
                    prev_y = Some(y);
                    prev_z = Some(z);
                }
            }

            let path2 = mesh
                .find_path(
                    (vehicle_x, vehicle_y, 0.5).into(),
                    (ARENA_WIDTH / 2.0 - 10.0, (ARENA_HEIGHT + UI_HEIGHT)/2.0, 0.5).into(),
                    nav_query_type,
                    nav_path_type,
                );

            if let Some(path) = path2 {
                let mut prev_x = None;
                let mut prev_y = None;
                let mut prev_z = None;

                for nav_vector in path.iter() {
                    let x = nav_vector.x;
                    let y = nav_vector.y;
                    let z = nav_vector.z;

                    if !prev_x.is_none() {
                        debug_lines_resource.draw_line(
                            [prev_x.unwrap(), prev_y.unwrap(), prev_z.unwrap()].into(),
                            [x, y, z].into(),
                            Srgba::new(0.2, 0.8, 1.0, 1.0),
                        );
                    }

                    prev_x = Some(x);
                    prev_y = Some(y);
                    prev_z = Some(z);
                }
            }

            let path3 = mesh
                .find_path(
                    (vehicle_x, vehicle_y, 0.5).into(),
                    (0.0, ARENA_HEIGHT, 0.5).into(),
                    nav_query_type,
                    nav_path_type,
                );

            if let Some(path) = path3 {
                let mut prev_x = None;
                let mut prev_y = None;
                let mut prev_z = None;

                for nav_vector in path.iter() {
                    let x = nav_vector.x;
                    let y = nav_vector.y;
                    let z = nav_vector.z;

                    if !prev_x.is_none() {
                        debug_lines_resource.draw_line(
                            [prev_x.unwrap(), prev_y.unwrap(), prev_z.unwrap()].into(),
                            [x, y, z].into(),
                            Srgba::new(0.5, 0.0, 1.0, 1.0),
                        );
                    }

                    prev_x = Some(x);
                    prev_y = Some(y);
                    prev_z = Some(z);
                }
            }
        }
    }
}