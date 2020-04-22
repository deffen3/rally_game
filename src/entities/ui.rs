use amethyst::{
    assets::{Loader},
    ecs::prelude::{Entity},
    prelude::*,
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};



/// ScoreBoard contains the actual score data
#[derive(Default)]
pub struct ScoreBoard {
    pub score_left: i32,
    pub score_right: i32,
}

/// ScoreText contains the ui text components that display the score
pub struct ScoreText {
    pub p1_score: Entity,
    pub p2_score: Entity,
}


/// Initialises the UI
pub fn initialise_ui(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );
    let p1_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        -50., -50., 1., 200., 50.,
    );
    let p2_transform = UiTransform::new(
        "P2".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        50., -50., 1., 200., 50.,
    );

    let p1_score = world
        .create_entity()
        .with(p1_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [1., 1., 1., 1.], 50.))
        .build();

    let p2_score = world
        .create_entity()
        .with(p2_transform)
        .with(UiText::new(font, "0".to_string(), [1., 1., 1., 1.], 50.))
        .build();

    world.insert(ScoreText { p1_score, p2_score });
}