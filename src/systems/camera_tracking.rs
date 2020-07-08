use amethyst::{
    core::{transform::Transform, Time},
    derive::SystemDesc,
    ecs::{Join, Read, ReadExpect, ReadStorage, System, SystemData, World, WriteStorage},
    renderer::camera::{Camera, Projection},
    window::ScreenDimensions,
};

use crate::components::{
    ArenaNames, ArenaProperties, ArenaStoreResource, Player, Vehicle, VehicleState,
};
use crate::resources::{GameModeSetup, GameModes};

const CAMERA_ZOOM_RATE: f32 = 120.0;
const CAMERA_TRANSLATE_MAX_RATE: f32 = 100.0;

#[derive(SystemDesc, Default)]
pub struct CameraTrackingSystem {
    pub arena_properties: ArenaProperties,
    pub init_state: bool,
}

impl<'s> System<'s> for CameraTrackingSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        WriteStorage<'s, Camera>,
        ReadExpect<'s, ScreenDimensions>,
        ReadExpect<'s, GameModeSetup>,
    );

    fn setup(&mut self, world: &mut World) {
        self.init_state = true;

        let arena_name;
        {
            let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();

            if let Some(game_mode_setup) = fetched_game_mode_setup {
                arena_name = game_mode_setup.arena_name.clone();
            } else {
                arena_name = ArenaNames::OpenEmptyMap;
            }
        }

        {
            let fetched_arena_store = world.try_fetch::<ArenaStoreResource>();

            if let Some(arena_store) = fetched_arena_store {
                self.arena_properties = match arena_store.properties.get(&arena_name) {
                    Some(arena_props_get) => (*arena_props_get).clone(),
                    _ => ArenaProperties::default(),
                };
            } else {
                self.arena_properties = ArenaProperties::default();
            }
        }
    }

    fn run(
        &mut self,
        (
        players,
        vehicles,
        mut transforms,
        time,
        mut cameras,
        screen_dimensions,
        game_mode_setup,
    ): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        let mut vehicle_xs = Vec::<f32>::new();
        let mut vehicle_ys = Vec::<f32>::new();

        for (_player, vehicle, transform) in (&players, &vehicles, &transforms).join() {
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
            vehicle_max_x = vehicle_xs[vehicle_xs.len() - 1];
            vehicle_min_y = vehicle_ys[0];
            vehicle_max_y = vehicle_ys[vehicle_ys.len() - 1];
        }

        //this is the extra buffer space that the camera gives
        let offset;
        if game_mode_setup.game_mode == GameModes::Race {
            offset = 160.0;
        } else {
            offset = 80.0;
        }

        vehicle_min_x = (vehicle_min_x - offset).max(0.0);
        vehicle_max_x = (vehicle_max_x + offset).min(self.arena_properties.width);
        vehicle_min_y = (vehicle_min_y - offset).max(-40.0);
        vehicle_max_y = (vehicle_max_y + offset).min(self.arena_properties.height);

        for (camera, transform) in (&mut cameras, &mut transforms).join() {
            let aspect_ratio = screen_dimensions.aspect_ratio();

            if self.init_state {
                self.init_state = false; //never goes back to true, until this system is re-dispatched

                //Standard full Arena translation
                transform.set_translation_x(self.arena_properties.width / 2.0);
                transform.set_translation_y(self.arena_properties.height / 2.0);

                //Standard full Arena Projection
                let x_delta = self.arena_properties.width;
                let y_delta = self.arena_properties.height;

                //keep aspect ratio consistent
                let target_delta = (x_delta / aspect_ratio).max(y_delta);

                let camera_left = -target_delta * aspect_ratio / 2.0;
                let camera_right = target_delta * aspect_ratio / 2.0;
                let camera_bottom = -target_delta / 2.0;
                let camera_top = target_delta / 2.0;

                camera.set_projection(Projection::orthographic(
                    camera_left,
                    camera_right,
                    camera_bottom,
                    camera_top,
                    0.0,
                    20.0,
                ));
            } else {
                //Update as game progresses

                let camera_projection = camera.projection().as_orthographic().unwrap();
                //let camera_left = camera_projection.left();
                //let camera_right = camera_projection.right();
                let camera_bottom = camera_projection.bottom();
                let camera_top = camera_projection.top();

                let camera_target_x = vehicle_min_x + (vehicle_max_x - vehicle_min_x) / 2.0;
                let camera_target_y = vehicle_min_y + (vehicle_max_y - vehicle_min_y) / 2.0;

                let x_delta = vehicle_max_x - vehicle_min_x;
                let y_delta = vehicle_max_y - vehicle_min_y;

                //keep aspect ratio consistent
                let target_delta = (x_delta / aspect_ratio).max(y_delta);

                let old_delta = camera_top - camera_bottom;
                let d_delta = (target_delta - old_delta)
                    .min(CAMERA_ZOOM_RATE)
                    .max(-CAMERA_ZOOM_RATE);

                let new_delta = old_delta + d_delta * dt;
                //let new_delta = target_delta;

                let camera_new_left = -new_delta * aspect_ratio / 2.0;
                let camera_new_right = new_delta * aspect_ratio / 2.0;
                let camera_new_bottom = -new_delta / 2.0;
                let camera_new_top = new_delta / 2.0;

                //Updated Projection
                camera.set_projection(Projection::orthographic(
                    camera_new_left,
                    camera_new_right,
                    camera_new_bottom,
                    camera_new_top,
                    0.0,
                    20.0,
                ));

                //Updated Translation
                let camera_x = transform.translation().x;
                let camera_y = transform.translation().y;

                let mut dx = (camera_target_x - camera_x)
                    .min(CAMERA_TRANSLATE_MAX_RATE)
                    .max(-CAMERA_TRANSLATE_MAX_RATE);
                if dx.abs() <= 0.01 {
                    dx = 0.0;
                }

                let mut dy = (camera_target_y - camera_y)
                    .min(CAMERA_TRANSLATE_MAX_RATE)
                    .max(-CAMERA_TRANSLATE_MAX_RATE);
                if dy.abs() <= 0.01 {
                    dy = 0.0;
                }

                transform.set_translation_x(camera_x + dx * dt);
                transform.set_translation_y(camera_y + dy * dt);
            }
        }
    }
}
