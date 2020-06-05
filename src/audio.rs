use amethyst::{
    assets::{AssetStorage, Loader},
    audio::{output::Output, OggFormat, Source, SourceHandle},
    ecs::{World, WorldExt},
};

const BOUNCE_SOUND: &str = "audio/vehicle_bounce.ogg";
const SCORE_SOUND: &str = "audio/weapon_hit.ogg";

pub struct Sounds {
    pub score_sfx: SourceHandle,
    pub bounce_sfx: SourceHandle,
}

/// Loads an ogg audio track.
fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

/// Initialise audio in the world. This will eventually include
/// the background tracks as well as the sound effects, but for now
/// we'll just work on sound effects.
pub fn initialize_audio(world: &mut World) {
    let sound_effects = {
        let loader = world.read_resource::<Loader>();

        Sounds {
            bounce_sfx: load_audio_track(&loader, &world, BOUNCE_SOUND),
            score_sfx: load_audio_track(&loader, &world, SCORE_SOUND),
        }
    };

    // Add sound effects to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    world.insert(sound_effects);
}

pub fn play_bounce_sound(sounds: &Sounds, storage: &AssetStorage<Source>, output: Option<&Output>) {
    if let Some(ref output) = output.as_ref() {
        if let Some(sound) = storage.get(&sounds.bounce_sfx) {
            output.play_once(sound, 0.05);
        }
    }
}

pub fn play_score_sound(sounds: &Sounds, storage: &AssetStorage<Source>, output: Option<&Output>) {
    if let Some(ref output) = output.as_ref() {
        if let Some(sound) = storage.get(&sounds.score_sfx) {
            output.play_once(sound, 0.05);
        }
    }
}
