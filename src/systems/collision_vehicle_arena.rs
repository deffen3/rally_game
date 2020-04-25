use amethyst::{
    core::{Transform, Time},
    derive::SystemDesc,
    ecs::{Join, Read, System, SystemData, WriteStorage, ReadStorage, ReadExpect, Entities},
    assets::AssetStorage,
    audio::{output::Output, Source},
};

use crate::components::{Vehicle, Player, kill_restart_vehicle, Hitbox, HitboxShape};

use crate::rally::{vehicle_damage_model, BASE_COLLISION_DAMAGE, 
    COLLISION_PIERCING_DAMAGE_PCT, COLLISION_SHIELD_DAMAGE_PCT,
    COLLISION_ARMOR_DAMAGE_PCT, COLLISION_HEALTH_DAMAGE_PCT};

use std::ops::Deref;
use crate::audio::{play_bounce_sound, Sounds};

use std::f32::consts::PI;



#[derive(SystemDesc, Default)]
pub struct CollisionVehToArenaSystem;



impl<'s> System<'s> for CollisionVehToArenaSystem {
    type SystemData = (
        WriteStorage<'s, Hitbox>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        Read<'s, Time>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(&mut self, (mut hitboxes, mut transforms, players, mut vehicles,
            time, storage, sounds, audio_output): Self::SystemData) {
        let dt = time.delta_seconds();

    }
}