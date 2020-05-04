use amethyst::{
    ecs::{Join, ReadStorage, System, SystemData, WriteStorage, Read, World},
    derive::SystemDesc,
    ui::UiText,
    core::Time,
};

use crate::components::{Player, Vehicle};
use crate::rally::{POINTS_TO_WIN, STOCK_LIVES, GAME_MODE, GameModes, MAX_PLAYERS, MATCH_TIME_LIMIT};

#[derive(SystemDesc, Default)]
pub struct VehicleStatusSystem {
    pub winners: Vec<usize>,
    pub losers: Vec<usize>,
    pub match_timer: f32,
}

impl<'s> System<'s> for VehicleStatusSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        WriteStorage<'s, UiText>,
        Read<'s, Time>,
    );

    fn setup(&mut self, _world: &mut World) {
        self.winners = vec![];
        self.losers = vec![];
        self.match_timer = 0.0;
    }

    fn run(&mut self, (players, vehicles, mut ui_text, time): Self::SystemData) {
        let dt = time.delta_seconds();

        self.match_timer += dt;

        //for (player, vehicle) in (players, vehicles).join() {
        for (player, vehicle) in (&players, &vehicles).join() {
            ui_text
                .get_mut(vehicle.player_status_text.shield)
                .unwrap()
                .text = format!("{:.0}", vehicle.shield.value.ceil());
                
            ui_text
                .get_mut(vehicle.player_status_text.armor)
                .unwrap()
                .text = format!("{:.0}", vehicle.armor.value.ceil());

            ui_text
                .get_mut(vehicle.player_status_text.health)
                .unwrap()
                .text = format!("{:.0}", vehicle.health.value.ceil());


            /*
            pub enum GameModes {
                ClassicGunGame, //First to get a kill with each weapon. Weapons are hot-swapped after kills.
                Deathmatch_Kills, //First to a certain number of kills. New weapons can be picked up from arena.
                Deathmatch_Stock, //If you run out of lives you are out. Last player alive wins. New weapons can be picked up from arena.
                Deathmatch_Timed_KD, //Match ends after set time. Kills-Deaths is winner. Self-destructs are minus 2 deaths. New weapons can be picked up from arena.
                KingOfTheHill, //Player gains points for being the only person in the special "hill" zone. First player to a certain number of points wins. New weapons can be picked up from arena.
            }*/

            //Scoring logic
            if MATCH_TIME_LIMIT < 0.0 || self.match_timer <= MATCH_TIME_LIMIT {
                let player_score;

                if GAME_MODE == GameModes::ClassicGunGame {
                    player_score = player.kills; //in this mode only the kills with the current weapon are counted.
                } else if GAME_MODE == GameModes::Deathmatch_Kills {
                    player_score = player.kills;
                } else if GAME_MODE == GameModes::Deathmatch_Stock {
                    player_score = STOCK_LIVES - player.deaths;
                } else if GAME_MODE == GameModes::Deathmatch_Timed_KD {
                    player_score = player.kills - player.deaths;
                } else if GAME_MODE == GameModes::KingOfTheHill {
                    player_score = 0;
                } else {
                    player_score = 0;
                }

                if GAME_MODE == GameModes::Deathmatch_Stock && (
                        player.deaths >= STOCK_LIVES || self.losers.len() > MAX_PLAYERS-1
                    ) {
                    if !self.losers.contains(&player.id) {
                        self.losers.push(player.id.clone());

                        let text_out = match self.losers.len() {
                            1 => "4th!".to_string(),
                            2 => "3rd!".to_string(),
                            3 => "2nd!".to_string(),
                            4 => "1st!".to_string(),
                            _ => "???".to_string(),
                        };

                        ui_text
                            .get_mut(vehicle.player_status_text.kills)
                            .unwrap()
                            .text = text_out;
                    }
                }
                else if POINTS_TO_WIN > 0 && player.kills >= POINTS_TO_WIN {
                    if !self.winners.contains(&player.id) {
                        self.winners.push(player.id.clone());

                        let text_out = match self.winners.len() {
                            1 => "1st!".to_string(),
                            2 => "2nd!".to_string(),
                            3 => "3rd!".to_string(),
                            4 => "4th!".to_string(),
                            _ => "???".to_string(),
                        };

                        ui_text
                            .get_mut(vehicle.player_status_text.kills)
                            .unwrap()
                            .text = text_out;
                    }
                }
                else {
                    ui_text
                        .get_mut(vehicle.player_status_text.kills)
                        .unwrap()
                        .text = format!("{:.0}", player_score);
                }
            }
        }
    }
}
