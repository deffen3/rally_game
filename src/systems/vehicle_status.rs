use amethyst::{
    ecs::{Join, ReadStorage, ReadExpect, WriteExpect, System, SystemData, WriteStorage, Read, World},
    derive::SystemDesc,
    ui::UiText,
    core::Time,
};

use crate::components::{Player, Vehicle};

use crate::resources::{GameModes, GameModeSetup, MatchTimer};

#[derive(SystemDesc, Default)]
pub struct VehicleStatusSystem {
    pub winners: Vec<usize>,
    pub losers: Vec<usize>,
}

impl<'s> System<'s> for VehicleStatusSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        WriteStorage<'s, UiText>,
        Read<'s, Time>,
        ReadExpect<'s, GameModeSetup>,
        WriteExpect<'s, MatchTimer>,
    );

    fn setup(&mut self, _world: &mut World) {
        self.winners = vec![];
        self.losers = vec![];
    }

    fn run(&mut self, (
            players,
            vehicles,
            mut ui_text,
            time,
            game_mode_setup,
            mut match_timer,
        ): Self::SystemData) {

        let dt = time.delta_seconds();

        //if no match time limit exists, or it does exist and timer is within the limit
        if game_mode_setup.match_time_limit < 0.0 || match_timer.time < game_mode_setup.match_time_limit {
            match_timer.time += dt;
        }

        //if match has a time limit, display time remaining
        if game_mode_setup.match_time_limit > 0.0  {
            ui_text
                .get_mut(match_timer.ui_entity)
                .unwrap()
                .text = format!("{:.0}", game_mode_setup.match_time_limit - match_timer.time);
        }
        else { //else display timer counting up
            ui_text
                .get_mut(match_timer.ui_entity)
                .unwrap()
                .text = format!("{:.0}", match_timer.time);
        }



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
                DeathmatchKills, //First to a certain number of kills. New weapons can be picked up from arena.
                DeathmatchStock, //If you run out of lives you are out. Last player alive wins. New weapons can be picked up from arena.
                DeathmatchTimedKD, //Match ends after set time. Kills-Deaths is winner. Self-destructs are minus 2 deaths. New weapons can be picked up from arena.
                KingOfTheHill, //Player gains points for being the only person in the special "hill" zone. First player to a certain number of points wins. New weapons can be picked up from arena.
            }*/

            //Scoring logic

            //if no match time limit exists, or it does exist and timer is within the limit
            if game_mode_setup.match_time_limit < 0.0 || match_timer.time <= game_mode_setup.match_time_limit {
                let player_score;

                if game_mode_setup.game_mode == GameModes::ClassicGunGame {
                    player_score = player.kills; //in this mode only the kills with the current weapon are counted.
                } else if game_mode_setup.game_mode == GameModes::DeathmatchKills {
                    player_score = player.kills;
                } else if game_mode_setup.game_mode == GameModes::DeathmatchStock {
                    player_score = game_mode_setup.stock_lives - player.deaths;
                } else if game_mode_setup.game_mode == GameModes::DeathmatchTimedKD {
                    player_score = player.kills - player.deaths;
                } else if game_mode_setup.game_mode == GameModes::Race {
                    player_score = player.laps_completed;
                } else if game_mode_setup.game_mode == GameModes::KingOfTheHill {
                    player_score = player.objective_points.floor() as i32;
                } else {
                    player_score = 0;
                }

                if game_mode_setup.game_mode == GameModes::DeathmatchStock && (
                        player.deaths >= game_mode_setup.stock_lives || self.losers.len() > game_mode_setup.max_players-1
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
                            .get_mut(vehicle.player_status_text.points)
                            .unwrap()
                            .text = text_out;
                    }
                }
                else if game_mode_setup.points_to_win > 0 && player_score >= game_mode_setup.points_to_win {
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
                            .get_mut(vehicle.player_status_text.points)
                            .unwrap()
                            .text = text_out;
                    }
                }
                else {
                    ui_text
                        .get_mut(vehicle.player_status_text.points)
                        .unwrap()
                        .text = format!("{:.0}", player_score);
                }
            }
        }
    }
}
