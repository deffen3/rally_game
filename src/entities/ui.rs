use amethyst::{
    assets::Loader,
    ecs::prelude::{Entity, Join},
    prelude::*,
    ui::{Anchor, TtfFormat, UiText, UiTransform, UiFinder},
    utils::removal::Removal,
};

use crate::resources::MatchTimer;

use crate::components::{Player, Vehicle};

use crate::resources::GameModeSetup;


pub fn initialize_timer_ui(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );

    //Match Timer
    let match_timer_transform = UiTransform::new(
        "MatchTimer".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        0.0,
        -10.0,
        1.,
        200.,
        50.,
    );

    let ui_entity = world
        .create_entity()
        .with(Removal::new(0 as u32))
        .with(match_timer_transform)
        .with(UiText::new(
            font.clone(),
            "0:00".to_string(),
            [1., 1., 1., 1.],
            50.,
        ))
        .build();

    world.insert(MatchTimer {
        time: 0.0,
        ui_entity,
    });
}

///contains the ui text components that display the player vehicle status
#[derive(Clone, Copy)]
pub struct PlayerStatusText {
    pub shield: Option<Entity>,
    pub armor: Option<Entity>,
    pub health: Option<Entity>,
    pub points: Option<Entity>,
}

/// Initialises the UI
pub fn connect_players_to_ui(world: &mut World) -> bool {
    // //Player status

    // let mut x = 700.;
    // let y = -960.;
    // let dy = 42.;
    // let dx = 80.;
    // let dx2 = 10.;


    // let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();

    // let max_players;
    // if let Some(game_mode_setup) = fetched_game_mode_setup {
    //     max_players = game_mode_setup.max_players;
    // } else {
    //     max_players = 4;
    // }

    let mut player_status_texts = [
        PlayerStatusText {
            shield: None,
            armor: None,
            health: None,
            points: None,
        },
        PlayerStatusText {
            shield: None,
            armor: None,
            health: None,
            points: None,
        },
        PlayerStatusText {
            shield: None,
            armor: None,
            health: None,
            points: None,
        },
        PlayerStatusText {
            shield: None,
            armor: None,
            health: None,
            points: None,
        },
    ];

    for player_index in 0..4 {
        world.exec(|finder: UiFinder<'_>| {
            let search_string = format!("p{}_shield", player_index+1);

            if let Some(entity) = finder.find(&search_string) {
                player_status_texts[player_index].shield = Some(entity);
            }
        });
        world.exec(|finder: UiFinder<'_>| {
            let search_string = format!("p{}_armor", player_index+1);

            if let Some(entity) = finder.find(&search_string) {
                player_status_texts[player_index].armor = Some(entity);
            }
        });
        world.exec(|finder: UiFinder<'_>| {
            let search_string = format!("p{}_health", player_index+1);

            if let Some(entity) = finder.find(&search_string) {
                player_status_texts[player_index].health = Some(entity);
            }
        });
        world.exec(|finder: UiFinder<'_>| {
            let search_string = format!("p{}_points", player_index+1);

            if let Some(entity) = finder.find(&search_string) {
                player_status_texts[player_index].points = Some(entity);
            }
        });
    }

    let players = world.read_storage::<Player>();
    let mut vehicles = world.write_storage::<Vehicle>();

    for (player, vehicle) in (&players, &mut vehicles).join() {
        vehicle.player_status_text = player_status_texts[player.id];
    }

    false
}
