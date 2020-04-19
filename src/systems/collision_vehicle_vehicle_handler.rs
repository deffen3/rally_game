use amethyst::core::{Transform, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, System, SystemData, WriteStorage, ReadStorage, Entities, Write};
use amethyst::shrev::{EventChannel, ReaderId};

use std::f32::consts::PI;

use crate::rally::{Vehicle, Player, CollisionEvent};

/*
#[derive(SystemDesc)]
pub struct CollisionVehHandlerSystem;
*/

#[derive(SystemDesc, Default)]
pub struct CollisionVehHandlerSystem{
    event_reader: Option<ReaderId<CollisionEvent>>,
}

pub const VEHICLE_HIT_BOUNCE_DECEL_PCT: f32 = -0.35;


impl<'s> System<'s> for CollisionVehHandlerSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        Read<'s, Time>,
        Write<'s, EventChannel<CollisionEvent>>,
    );


    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.event_reader = Some(world.fetch_mut::<EventChannel<CollisionEvent>>().register_reader());
    }

    fn run(&mut self, (entities, transforms, players, mut vehicles, time, mut collision_event_channel): Self::SystemData) {
        //let dt = time.delta_seconds();

        for (vehicle_1_entity, vehicle_1, player_1, vehicle_1_transform) in (&*entities, &vehicles, &players, &transforms).join() {
            
        }
    }
}