use amethyst::{
    core::{
        Time,
        transform::{Transform},
    },
    derive::SystemDesc,
    ecs::{System, Join, SystemData, WriteStorage, ReadStorage, ReadExpect, Read},
    window::ScreenDimensions,
    renderer::{
        camera::{Camera, Projection},
    },
};


use crate::components::{Vehicle, Player, VehicleState};

use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH};


const CAMERA_ZOOM_RATE: f32 = 0.01;
const CAMERA_TRANSLATE_MAX_RATE: f32 = 80.0;


#[derive(SystemDesc)]
pub struct CameraTrackingSystem;

impl<'s> System<'s> for CameraTrackingSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        WriteStorage<'s, Camera>,
        ReadExpect<'s, ScreenDimensions>,
    );

    fn run(
        &mut self, (
        players,
        vehicles,
        mut transforms,
        time,
        mut cameras,
        screen_dimensions,
    ): Self::SystemData
    ) {
        let dt = time.delta_seconds();

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
            
            let aspect_ratio = screen_dimensions.aspect_ratio();

            //Standard full Arena Projection
            // camera.set_projection(Projection::orthographic(
            //     -ARENA_WIDTH/2.0,
            //     ARENA_WIDTH/2.0,
            //     -ARENA_HEIGHT/2.0,
            //     ARENA_HEIGHT/2.0,
            //     0.0,
            //     20.0,
            // ));

            let camera_projection = camera.projection().as_orthographic().unwrap();
            let camera_left = camera_projection.left();
            let camera_right = camera_projection.right();
            let camera_top = camera_projection.top();
            let camera_bottom = camera_projection.bottom();

            let camera_target_x = vehicle_min_x + (vehicle_max_x - vehicle_min_x)/2.0;
            let camera_target_y = vehicle_min_y + (vehicle_max_y - vehicle_min_y)/2.0;

            let x_delta = vehicle_max_x - vehicle_min_x;
            let y_delta = vehicle_max_y - vehicle_min_y;

            //keep aspect ratio consistent
            let max_delta = (x_delta/aspect_ratio).max(y_delta);

            let camera_target_left = -max_delta*aspect_ratio/2.0;
            let camera_target_right = max_delta*aspect_ratio/2.0;
            let camera_target_top = -max_delta/2.0;
            let camera_target_bottom = max_delta/2.0;

            let camera_new_left = camera_left + (camera_target_left - camera_left);
            let camera_new_right = camera_right + (camera_target_right - camera_right);
            let camera_new_top = camera_top + (camera_target_top - camera_top);
            let camera_new_bottom = camera_bottom + (camera_target_bottom - camera_bottom);

            camera.set_projection(Projection::orthographic(
                camera_new_left,
                camera_new_right,
                camera_new_top,
                camera_new_bottom,
                0.0,
                20.0,
            ));


            let camera_x = transform.translation().x;
            let camera_y = transform.translation().y;

            //Standard full Arena translation
            // transform.set_translation_x(ARENA_WIDTH/2.0);
            // transform.set_translation_y(ARENA_HEIGHT/2.0);

            let mut dx = (camera_target_x - camera_x).min(CAMERA_TRANSLATE_MAX_RATE).max(-CAMERA_TRANSLATE_MAX_RATE);
            if dx.abs() <= 0.01 {
                dx = 0.0;
            }

            let mut dy = (camera_target_y - camera_y).min(CAMERA_TRANSLATE_MAX_RATE).max(-CAMERA_TRANSLATE_MAX_RATE);
            if dy.abs() <= 0.01 {
                dy = 0.0;
            }

            transform.set_translation_x(camera_x + dx*dt);
            //transform.set_translation_x(camera_target_x);
            transform.set_translation_y(camera_y + dy*dt);
            //transform.set_translation_y(camera_target_y);

            //log::info!("{} {}", camera_x, camera_y);
            //log::info!("{} {} {} {}", camera_left, camera_right, camera_top, camera_bottom);
        }
    }
}