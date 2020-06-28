use amethyst::{
    core::{
        transform::{Transform},
    },
    derive::SystemDesc,
    ecs::{System, Join, SystemData, WriteStorage, ReadStorage},
    renderer::{
        camera::{Camera, Projection},
    },
};


use crate::components::{Vehicle, Player, VehicleState};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};


#[derive(SystemDesc)]
pub struct CameraTrackingSystem;

impl<'s> System<'s> for CameraTrackingSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Camera>,
    );

    fn run(
        &mut self, (
        players,
        vehicles,
        mut transforms,
        mut cameras,
    ): Self::SystemData
    ) {
        let mut vehicle_xs = Vec::<f32>::new();
        let mut vehicle_ys = Vec::<f32>::new();

        for (_player, vehicle, transform) in (&players, &vehicles, &transforms).join()
        {
            if vehicle.state == VehicleState::Active || vehicle.state == VehicleState::InRespawn {
                let vehicle_x = transform.translation().x;
                let vehicle_y = transform.translation().y;

                vehicle_xs.push(vehicle_x);
                vehicle_ys.push(vehicle_y);
            }            
        }

        vehicle_xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        vehicle_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut vehicle_min_x: f32 = 0.0;
        let mut vehicle_max_x: f32 = 0.0;
        let mut vehicle_min_y: f32 = 0.0;
        let mut vehicle_max_y: f32 = 0.0;

        if vehicle_xs.len() > 0 {
            vehicle_min_x = vehicle_xs[0];
            vehicle_max_x = vehicle_xs[vehicle_xs.len()-1];
            vehicle_min_y = vehicle_ys[0];
            vehicle_max_y = vehicle_ys[vehicle_ys.len()-1];
        }

        let offset = 80.0;

        vehicle_min_x = (vehicle_min_x - offset).max(0.0);
        vehicle_max_x = (vehicle_max_x + offset).min(ARENA_WIDTH);
        vehicle_min_y = (vehicle_min_y - offset).max(0.0);
        vehicle_max_y = (vehicle_max_y + offset).min(ARENA_HEIGHT);

        for (camera, transform) in (&mut cameras, &mut transforms).join() {
            let camera_x = transform.translation().x;
            let camera_y = transform.translation().y;

            let camera_projection = camera.projection();

            //log::info!("{} {} {:?}", camera_x, camera_y, camera_projection);

            //Standard Projection
            // camera.set_projection(Projection::orthographic(
            //     -ARENA_WIDTH/2.0,
            //     ARENA_WIDTH/2.0,
            //     -ARENA_HEIGHT/2.0,
            //     ARENA_HEIGHT/2.0,
            //     0.0,
            //     20.0,
            // ));

            let aspect_ratio = 16.0/9.0;

            let x_delta = vehicle_max_x - vehicle_min_x;
            let y_delta = vehicle_max_y - vehicle_min_y;

            let max_delta = (x_delta/aspect_ratio).max(y_delta);

            camera.set_projection(Projection::orthographic(
                -max_delta*aspect_ratio/2.0,
                max_delta*aspect_ratio/2.0,
                -max_delta/2.0,
                max_delta/2.0,
                0.0,
                20.0,
            ));
            

            transform.set_translation_x(vehicle_min_x + (vehicle_max_x - vehicle_min_x)/2.0);
            transform.set_translation_y(vehicle_min_y + (vehicle_max_y - vehicle_min_y)/2.0);
        }
    }
}