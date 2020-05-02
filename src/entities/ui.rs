use amethyst::{
    assets::Loader,
    ecs::prelude::Entity,
    prelude::*,
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};

///contains the ui text components that display the player vehicle status
#[derive(Clone, Copy)]
pub struct PlayerStatusText {
    pub shield: Entity,
    pub armor: Entity,
    pub health: Entity,
    pub kills: Entity,
}


/// Initialises the UI
pub fn initialize_ui(world: &mut World) -> [PlayerStatusText; 4] {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );

    let mut x = -450.;
    let y = -960.;
    let dy = 42.;
    let dx = 80.;
    let dx2 = 10.;

    let p1_shield_transform = UiTransform::new(
        "P1".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    x += dx;
    let p1_armor_transform = UiTransform::new(
        "P1".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    let p1_kills_transform = UiTransform::new(
        "P1".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y + dy,
        1.,
        200.,
        50.,
    );
    x += dx;
    let p1_health_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    x += dx + dx2;
    let p2_shield_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    x += dx;
    let p2_armor_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    let p2_kills_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y + dy,
        1.,
        200.,
        50.,
    );
    x += dx;
    let p2_health_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    x += dx + dx2;
    let p3_shield_transform = UiTransform::new(
        "P3".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    x += dx;
    let p3_armor_transform = UiTransform::new(
        "P3".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    let p3_kills_transform = UiTransform::new(
        "P3".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y + dy,
        1.,
        200.,
        50.,
    );
    x += dx;
    let p3_health_transform = UiTransform::new(
        "P3".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    x += dx + dx2;
    let p4_shield_transform = UiTransform::new(
        "P4".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    x += dx;
    let p4_armor_transform = UiTransform::new(
        "P4".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );
    let p4_kills_transform = UiTransform::new(
        "P3".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y + dy,
        1.,
        200.,
        50.,
    );
    x += dx;
    let p4_health_transform = UiTransform::new(
        "P4".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        x,
        y,
        1.,
        200.,
        50.,
    );

    let p1_shield = world
        .create_entity()
        .with(p1_shield_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [0., 0., 1., 1.],
            50.,
        ))
        .build();

    let p1_armor = world
        .create_entity()
        .with(p1_armor_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [0., 0., 0., 1.],
            50.,
        ))
        .build();

    let p1_health = world
        .create_entity()
        .with(p1_health_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1., 0., 0., 1.],
            50.,
        ))
        .build();

    let p1_kills = world
        .create_entity()
        .with(p1_kills_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1., 1., 1., 1.],
            50.,
        ))
        .build();

    let p2_shield = world
        .create_entity()
        .with(p2_shield_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [0., 0., 1., 1.],
            50.,
        ))
        .build();

    let p2_armor = world
        .create_entity()
        .with(p2_armor_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [0., 0., 0., 1.],
            50.,
        ))
        .build();

    let p2_health = world
        .create_entity()
        .with(p2_health_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1., 0., 0., 1.],
            50.,
        ))
        .build();

    let p2_kills = world
        .create_entity()
        .with(p2_kills_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1., 1., 1., 1.],
            50.,
        ))
        .build();

    let p3_shield = world
        .create_entity()
        .with(p3_shield_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [0., 0., 1., 1.],
            50.,
        ))
        .build();

    let p3_armor = world
        .create_entity()
        .with(p3_armor_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [0., 0., 0., 1.],
            50.,
        ))
        .build();

    let p3_health = world
        .create_entity()
        .with(p3_health_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1., 0., 0., 1.],
            50.,
        ))
        .build();

    let p3_kills = world
        .create_entity()
        .with(p3_kills_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1., 1., 1., 1.],
            50.,
        ))
        .build();

    let p4_shield = world
        .create_entity()
        .with(p4_shield_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [0., 0., 1., 1.],
            50.,
        ))
        .build();

    let p4_armor = world
        .create_entity()
        .with(p4_armor_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [0., 0., 0., 1.],
            50.,
        ))
        .build();

    let p4_health = world
        .create_entity()
        .with(p4_health_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1., 0., 0., 1.],
            50.,
        ))
        .build();

    let p4_kills = world
        .create_entity()
        .with(p4_kills_transform)
        .with(UiText::new(
            font,
            "0".to_string(),
            [1., 1., 1., 1.],
            50.,
        ))
        .build();

    [
        PlayerStatusText {
            shield: p1_shield,
            armor: p1_armor,
            health: p1_health,
            kills: p1_kills,
        },
        PlayerStatusText {
            shield: p2_shield,
            armor: p2_armor,
            health: p2_health,
            kills: p2_kills,
        },
        PlayerStatusText {
            shield: p3_shield,
            armor: p3_armor,
            health: p3_health,
            kills: p3_kills,
        },
        PlayerStatusText {
            shield: p4_shield,
            armor: p4_armor,
            health: p4_health,
            kills: p4_kills,
        },
    ]
}
