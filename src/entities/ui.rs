use amethyst::{
    assets::{Loader},
    ecs::prelude::{Entity},
    prelude::*,
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};



/// ScoreText contains the ui text components that display the score
pub struct ScoreText {
    pub p1_shield: Entity,
    pub p2_shield: Entity,
    pub p3_shield: Entity,
    pub p4_shield: Entity,
    pub p1_armor: Entity,
    pub p2_armor: Entity,
    pub p3_armor: Entity,
    pub p4_armor: Entity,
    pub p1_health: Entity,
    pub p2_health: Entity,
    pub p3_health: Entity,
    pub p4_health: Entity,
}




/// Initialises the UI
pub fn initialise_ui(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );


    let mut x = -450.;
    let y = -950.;

    let p1_shield_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p1_armor_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p1_health_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p2_shield_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p2_armor_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p2_health_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p3_shield_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p3_armor_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p3_health_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p4_shield_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p4_armor_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );
    x += 80.;
    let p4_health_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        x, y, 1., 200., 50.,
    );

    let p1_shield = world
        .create_entity()
        .with(p1_shield_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [0., 0., 1., 1.], 50.))
        .build();

    let p1_armor = world
        .create_entity()
        .with(p1_armor_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [0., 0., 0., 1.], 50.))
        .build();

    let p1_health = world
        .create_entity()
        .with(p1_health_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [1., 0., 0., 1.], 50.))
        .build();

    let p2_shield = world
        .create_entity()
        .with(p2_shield_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [0., 0., 1., 1.], 50.))
        .build();

    let p2_armor = world
        .create_entity()
        .with(p2_armor_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [0., 0., 0., 1.], 50.))
        .build();

    let p2_health = world
        .create_entity()
        .with(p2_health_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [1., 0., 0., 1.], 50.))
        .build();

    let p3_shield = world
        .create_entity()
        .with(p3_shield_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [0., 0., 1., 1.], 50.))
        .build();

    let p3_armor = world
        .create_entity()
        .with(p3_armor_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [0., 0., 0., 1.], 50.))
        .build();

    let p3_health = world
        .create_entity()
        .with(p3_health_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [1., 0., 0., 1.], 50.))
        .build();

    let p4_shield = world
        .create_entity()
        .with(p4_shield_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [0., 0., 1., 1.], 50.))
        .build();

    let p4_armor = world
        .create_entity()
        .with(p4_armor_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [0., 0., 0., 1.], 50.))
        .build();

    let p4_health = world
        .create_entity()
        .with(p4_health_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [1., 0., 0., 1.], 50.))
        .build();

    world.insert(ScoreText { p1_shield, p2_shield, p3_shield, p4_shield,
        p1_armor, p2_armor, p3_armor, p4_armor,
        p1_health, p2_health, p3_health, p4_health
    });
}