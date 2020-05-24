use amethyst::{
    core::{Time},
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder, UiText},
    winit::VirtualKeyCode,
    utils::{
        removal::{exec_removal},
    },
};

use crate::menu::MainMenu;

use crate::resources::{GameScore};


const SCORE_SCREEN_TIMER_INIT: f32 = 1.0;

const BUTTON_BACK_TO_MENU: &str = "back_to_menu";



const P1_TITLE: &str = "p1_title";
const P2_TITLE: &str = "p2_title";
const P3_TITLE: &str = "p3_title";
const P4_TITLE: &str = "p4_title";

const P1_PLACE: &str = "p1_place";
const P2_PLACE: &str = "p2_place";
const P3_PLACE: &str = "p3_place";
const P4_PLACE: &str = "p4_place";

const P1_SCORE: &str = "p1_score";
const P2_SCORE: &str = "p2_score";
const P3_SCORE: &str = "p3_score";
const P4_SCORE: &str = "p4_score";

const P1_KILLS: &str = "p1_kills";
const P2_KILLS: &str = "p2_kills";
const P3_KILLS: &str = "p3_kills";
const P4_KILLS: &str = "p4_kills";

const P1_DEATHS: &str = "p1_deaths";
const P2_DEATHS: &str = "p2_deaths";
const P3_DEATHS: &str = "p3_deaths";
const P4_DEATHS: &str = "p4_deaths";

const P1_TIMER: &str = "p1_timer";
const P2_TIMER: &str = "p2_timer";
const P3_TIMER: &str = "p3_timer";
const P4_TIMER: &str = "p4_timer";



#[derive(Default, Debug)]
pub struct ScoreScreen {
    ui_root: Option<Entity>,
    
    button_back_to_menu: Option<Entity>,

    p1_title: Option<Entity>,
    p2_title: Option<Entity>,
    p3_title: Option<Entity>,
    p4_title: Option<Entity>,

    p1_place: Option<Entity>,
    p2_place: Option<Entity>,
    p3_place: Option<Entity>,
    p4_place: Option<Entity>,

    p1_score: Option<Entity>,
    p2_score: Option<Entity>,
    p3_score: Option<Entity>,
    p4_score: Option<Entity>,

    p1_kills: Option<Entity>,
    p2_kills: Option<Entity>,
    p3_kills: Option<Entity>,
    p4_kills: Option<Entity>,

    p1_deaths: Option<Entity>,
    p2_deaths: Option<Entity>,
    p3_deaths: Option<Entity>,
    p4_deaths: Option<Entity>,

    p1_timer: Option<Entity>,
    p2_timer: Option<Entity>,
    p3_timer: Option<Entity>,
    p4_timer: Option<Entity>,


    load_timer: f32,
    loaded: bool,
}

impl SimpleState for ScoreScreen {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        {
            let fetched_game_score = world.try_fetch_mut::<GameScore>();

            if let Some(mut game_score) = fetched_game_score {
                game_score.game_ended = false; //reset
            }
        }

        self.loaded = false;
        self.load_timer = SCORE_SCREEN_TIMER_INIT;
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // only search for buttons if they have not been found yet
        let StateData { world, .. } = state_data;

        let dt;
        {
            let fetched_time = world.try_fetch::<Time>();

            if let Some(time) = fetched_time {
                dt = time.delta_seconds();
            }
            else {
                dt = 0.01;
            }
        }

        {
            if !self.loaded && self.load_timer <= 0.0 {
                self.loaded = true;
                self.ui_root =
                    Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/score_screen.ron", ())));
            }
            else {
                self.load_timer -= dt;
            }
        }

        {
            if self.button_back_to_menu.is_none() ||
                self.p1_title.is_none() ||
                self.p2_title.is_none() ||
                self.p3_title.is_none() ||
                self.p4_title.is_none() ||
                self.p1_place.is_none() ||
                self.p2_place.is_none() ||
                self.p3_place.is_none() ||
                self.p4_place.is_none() ||
                self.p1_score.is_none() ||
                self.p2_score.is_none() ||
                self.p3_score.is_none() ||
                self.p4_score.is_none() ||
                self.p1_kills.is_none() ||
                self.p2_kills.is_none() ||
                self.p3_kills.is_none() ||
                self.p4_kills.is_none() ||
                self.p1_deaths.is_none() ||
                self.p2_deaths.is_none() ||
                self.p3_deaths.is_none() ||
                self.p4_deaths.is_none() ||
                self.p1_timer.is_none() ||
                self.p2_timer.is_none() ||
                self.p3_timer.is_none() ||
                self.p4_timer.is_none()
            {
                world.exec(|ui_finder: UiFinder<'_>| {
                    self.button_back_to_menu = ui_finder.find(BUTTON_BACK_TO_MENU);
                    self.p1_title = ui_finder.find(P1_TITLE);
                    self.p2_title = ui_finder.find(P2_TITLE);
                    self.p3_title = ui_finder.find(P3_TITLE);
                    self.p4_title = ui_finder.find(P4_TITLE);
                    self.p1_place = ui_finder.find(P1_PLACE);
                    self.p2_place = ui_finder.find(P2_PLACE);
                    self.p3_place = ui_finder.find(P3_PLACE);
                    self.p4_place = ui_finder.find(P4_PLACE);
                    self.p1_score = ui_finder.find(P1_SCORE);
                    self.p2_score = ui_finder.find(P2_SCORE);
                    self.p3_score = ui_finder.find(P3_SCORE);
                    self.p4_score = ui_finder.find(P4_SCORE);
                    self.p1_kills = ui_finder.find(P1_KILLS);
                    self.p2_kills = ui_finder.find(P2_KILLS);
                    self.p3_kills = ui_finder.find(P3_KILLS);
                    self.p4_kills = ui_finder.find(P4_KILLS);
                    self.p1_deaths = ui_finder.find(P1_DEATHS);
                    self.p2_deaths = ui_finder.find(P2_DEATHS);
                    self.p3_deaths = ui_finder.find(P3_DEATHS);
                    self.p4_deaths = ui_finder.find(P4_DEATHS);
                    self.p1_timer = ui_finder.find(P1_TIMER);
                    self.p2_timer = ui_finder.find(P2_TIMER);
                    self.p3_timer = ui_finder.find(P3_TIMER);
                    self.p4_timer = ui_finder.find(P4_TIMER);
                });
            }
        }


        let mut ui_text = world.write_storage::<UiText>();
        let fetched_game_score = world.try_fetch::<GameScore>();

        if let Some(game_score) = fetched_game_score {
            if let Some(p1_title) = self.p1_title.and_then(|entity| ui_text.get_mut(entity)) {
                p1_title.text = get_title_text(game_score.placements[0].0);
            }
            if let Some(p2_title) = self.p2_title.and_then(|entity| ui_text.get_mut(entity)) {
                p2_title.text = get_title_text(game_score.placements[1].0);
            }
            if let Some(p3_title) = self.p3_title.and_then(|entity| ui_text.get_mut(entity)) {
                p3_title.text = get_title_text(game_score.placements[2].0);
            }
            if let Some(p4_title) = self.p4_title.and_then(|entity| ui_text.get_mut(entity)) {
                p4_title.text = get_title_text(game_score.placements[3].0);
            }

            if let Some(p1_place) = self.p1_place.and_then(|entity| ui_text.get_mut(entity)) {
                p1_place.text = get_placement_text(game_score.placements[0].1);
            }
            if let Some(p2_place) = self.p2_place.and_then(|entity| ui_text.get_mut(entity)) {
                p2_place.text = get_placement_text(game_score.placements[1].1);
            }
            if let Some(p3_place) = self.p3_place.and_then(|entity| ui_text.get_mut(entity)) {
                p3_place.text = get_placement_text(game_score.placements[2].1);
            }
            if let Some(p4_place) = self.p4_place.and_then(|entity| ui_text.get_mut(entity)) {
                p4_place.text = get_placement_text(game_score.placements[3].1);
            }

            if let Some(p1_score) = self.p1_score.and_then(|entity| ui_text.get_mut(entity)) {
                p1_score.text = game_score.placements[0].2.to_string();
            }
            if let Some(p2_score) = self.p2_score.and_then(|entity| ui_text.get_mut(entity)) {
                p2_score.text = game_score.placements[1].2.to_string();
            }
            if let Some(p3_score) = self.p3_score.and_then(|entity| ui_text.get_mut(entity)) {
                p3_score.text = game_score.placements[2].2.to_string();
            }
            if let Some(p4_score) = self.p4_score.and_then(|entity| ui_text.get_mut(entity)) {
                p4_score.text = game_score.placements[3].2.to_string();
            }

            if let Some(p1_kills) = self.p1_kills.and_then(|entity| ui_text.get_mut(entity)) {
                p1_kills.text = game_score.placements[0].3.to_string();
            }
            if let Some(p2_kills) = self.p2_kills.and_then(|entity| ui_text.get_mut(entity)) {
                p2_kills.text = game_score.placements[1].3.to_string();
            }
            if let Some(p3_kills) = self.p3_kills.and_then(|entity| ui_text.get_mut(entity)) {
                p3_kills.text = game_score.placements[2].3.to_string();
            }
            if let Some(p4_kills) = self.p4_kills.and_then(|entity| ui_text.get_mut(entity)) {
                p4_kills.text = game_score.placements[3].3.to_string();
            }

            if let Some(p1_deaths) = self.p1_deaths.and_then(|entity| ui_text.get_mut(entity)) {
                p1_deaths.text = game_score.placements[0].4.to_string();
            }
            if let Some(p2_deaths) = self.p2_deaths.and_then(|entity| ui_text.get_mut(entity)) {
                p2_deaths.text = game_score.placements[1].4.to_string();
            }
            if let Some(p3_deaths) = self.p3_deaths.and_then(|entity| ui_text.get_mut(entity)) {
                p3_deaths.text = game_score.placements[2].4.to_string();
            }
            if let Some(p4_deaths) = self.p4_deaths.and_then(|entity| ui_text.get_mut(entity)) {
                p4_deaths.text = game_score.placements[3].4.to_string();
            }

            if let Some(p1_timer) = self.p1_timer.and_then(|entity| ui_text.get_mut(entity)) {
                let match_time_seconds: i32 = game_score.placements[0].5.floor() as i32 % 60;
                let match_time_minutes: i32 = game_score.placements[0].5.floor() as i32 / 60;

                p1_timer.text = format!("{:.0}:{:0>2.0}", match_time_minutes, match_time_seconds);
            }
            if let Some(p2_timer) = self.p2_timer.and_then(|entity| ui_text.get_mut(entity)) {
                let match_time_seconds: i32 = game_score.placements[1].5.floor() as i32 % 60;
                let match_time_minutes: i32 = game_score.placements[1].5.floor() as i32 / 60;

                p2_timer.text = format!("{:.0}:{:0>2.0}", match_time_minutes, match_time_seconds);
            }
            if let Some(p3_timer) = self.p3_timer.and_then(|entity| ui_text.get_mut(entity)) {
                let match_time_seconds: i32 = game_score.placements[2].5.floor() as i32 % 60;
                let match_time_minutes: i32 = game_score.placements[2].5.floor() as i32 / 60;

                p3_timer.text = format!("{:.0}:{:0>2.0}", match_time_minutes, match_time_seconds);
            }
            if let Some(p4_timer) = self.p4_timer.and_then(|entity| ui_text.get_mut(entity)) {
                let match_time_seconds: i32 = game_score.placements[3].5.floor() as i32 % 60;
                let match_time_minutes: i32 = game_score.placements[3].5.floor() as i32 / 60;

                p4_timer.text = format!("{:.0}:{:0>2.0}", match_time_minutes, match_time_seconds);
            }

        }

        Trans::None
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {

        match event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Switch] Switching back to MainMenu!");
                    Trans::Switch(Box::new(MainMenu::default()))
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(UiEvent {
                event_type: UiEventType::Click,
                target,
            }) => {
                if Some(target) == self.button_back_to_menu {
                    log::info!("[Trans::Switch] Switching back to MainMenu!");
                    return Trans::Switch(Box::new(MainMenu::default()));
                }

                Trans::None
            }
            _ => Trans::None,
        }
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        // after destroying the current UI, invalidate references as well (makes things cleaner)
        if let Some(root_entity) = self.ui_root {
            data.world
                .delete_entity(root_entity)
                .expect("Failed to remove CustomArenaMenu");
        }

        exec_removal(&data.world.entities(), &data.world.read_storage(), 0 as u32);

        self.ui_root = None;

        self.button_back_to_menu = None;

        self.p1_title = None;
        self.p2_title = None;
        self.p3_title = None;
        self.p4_title = None;
        self.p1_place = None;
        self.p2_place = None;
        self.p3_place = None;
        self.p4_place = None;
        self.p1_score = None;
        self.p2_score = None;
        self.p3_score = None;
        self.p4_score = None;
        self.p1_kills = None;
        self.p2_kills = None;
        self.p3_kills = None;
        self.p4_kills = None;
        self.p1_deaths = None;
        self.p2_deaths = None;
        self.p3_deaths = None;
        self.p4_deaths = None;
        self.p1_timer = None;
        self.p2_timer = None;
        self.p3_timer = None;
        self.p4_timer = None;
    }
}

fn get_title_text(player_in: usize) -> String {
    let title_text = match player_in {
        0 => "PLAYER 1".to_string(),
        1 => "PLAYER 2".to_string(),
        2 => "PLAYER 3".to_string(),
        3 => "PLAYER 4".to_string(),
        _ => "PLAYER ?".to_string(),
    };

    title_text
}

fn get_placement_text(place_in: i32) -> String {
    let place_text = match place_in {
        1 => "1st".to_string(),
        2 => "2nd".to_string(),
        3 => "3rd".to_string(),
        4 => "4th".to_string(),
        _ => "???".to_string(),
    };

    place_text
}