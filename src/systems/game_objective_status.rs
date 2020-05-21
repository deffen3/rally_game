use amethyst::{
    core::Time,
    derive::SystemDesc,
    ecs::{
        Join, Read, ReadExpect, ReadStorage, System, SystemData, World, WriteExpect, WriteStorage,
    },
    ui::UiText,
};

use crate::components::{Player, Vehicle};

use crate::resources::{GameModeSetup, GameModes, MatchTimer, GameScore, GameEndCondition};

#[derive(SystemDesc, Default)]
pub struct VehicleStatusSystem {
    pub winners: Vec<usize>,
    pub losers: Vec<usize>,
    pub game_end_wait_timer: f32,
    pub scores: [i32; 4],
    pub placements: [i32; 4],
}

impl<'s> System<'s> for VehicleStatusSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        WriteStorage<'s, UiText>,
        Read<'s, Time>,
        ReadExpect<'s, GameModeSetup>,
        WriteExpect<'s, GameScore>,
        WriteExpect<'s, MatchTimer>,
    );

    fn setup(&mut self, _world: &mut World) {
        self.winners = vec![];
        self.losers = vec![];
        self.scores = [0; 4];
        self.placements = [0; 4];
    }

    fn run(
        &mut self,
        (players, vehicles, mut ui_text, time, game_mode_setup, mut game_score, mut match_timer): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        //if no match time limit exists, or it does exist and timer is within the limit
        if game_mode_setup.match_time_limit < 0.0
            || match_timer.time < game_mode_setup.match_time_limit
        {
            match_timer.time += dt;
        }

        let match_time: f32;

        //if match has a time limit, display time remaining
        if game_mode_setup.match_time_limit > 0.0 {
            if game_mode_setup.match_time_limit - match_timer.time < 0.0 {
                match_time = 0.0;
            } else {
                match_time = game_mode_setup.match_time_limit - match_timer.time;
            }
        } else {
            //else display timer counting up
            match_time = match_timer.time;
        }

        let match_time_seconds: i32 = match_time.floor() as i32 % 60;
        let match_time_minutes: i32 = match_time.floor() as i32 / 60;

        ui_text.get_mut(match_timer.ui_entity).unwrap().text =
            format!("{:.0}:{:0>2.0}", match_time_minutes, match_time_seconds);

        //for (player, vehicle) in (players, vehicles).join() {
        for (player, vehicle) in (&players, &vehicles).join() {

            if let Some(shield_status) = vehicle.player_status_text.shield {
                ui_text
                    .get_mut(shield_status)
                    .unwrap()
                    .text = format!("{:.0}", vehicle.shield.value.ceil());
            }

            if let Some(armor_status) = vehicle.player_status_text.armor {
                ui_text
                    .get_mut(armor_status)
                    .unwrap()
                    .text = format!("{:.0}", vehicle.armor.value.ceil());
            }
            
            if let Some(health_status) = vehicle.player_status_text.health {
                ui_text
                    .get_mut(health_status)
                    .unwrap()
                    .text = format!("{:.0}", vehicle.health.value.ceil());
            }

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
            if game_mode_setup.match_time_limit < 0.0
                || match_timer.time <= game_mode_setup.match_time_limit
            {
                let displayed_player_score = match game_mode_setup.game_mode {
                    GameModes::ClassicGunGame => player.kills, //in this mode only the kills with the current weapon are counted.
                    GameModes::DeathmatchKills => player.kills,
                    GameModes::DeathmatchStock => game_mode_setup.stock_lives - player.deaths,
                    GameModes::DeathmatchTimedKD => player.kills - player.deaths,
                    GameModes::Race => player.laps_completed,
                    GameModes::KingOfTheHill => player.objective_points.floor() as i32,
                };

                self.scores[player.id.clone()] = displayed_player_score;

                if game_mode_setup.game_mode == GameModes::DeathmatchStock
                    && (player.deaths >= game_mode_setup.stock_lives
                        || self.losers.len() > game_mode_setup.max_players - 1)
                {
                    if !self.losers.contains(&player.id) {
                        self.losers.push(player.id.clone());

                        let (text_out, place) = match self.losers.len() {
                            1 => ("4th!".to_string(), 4),
                            2 => ("3rd!".to_string(), 3),
                            3 => ("2nd!".to_string(), 2),
                            4 => ("1st!".to_string(), 1),
                            _ => ("???".to_string(), 0),
                        };

                        self.placements[player.id.clone()] = place;

                        if let Some(points_status) = vehicle.player_status_text.points {
                            ui_text
                                .get_mut(points_status)
                                .unwrap()
                                .text = text_out;
                        }
                    }
                } else if game_mode_setup.points_to_win > 0
                    && displayed_player_score >= game_mode_setup.points_to_win
                {
                    if !self.winners.contains(&player.id) {
                        self.winners.push(player.id.clone());

                        let (text_out, place) = match self.winners.len() {
                            1 => ("1st!".to_string(), 1),
                            2 => ("2nd!".to_string(), 2),
                            3 => ("3rd!".to_string(), 3),
                            4 => ("4th!".to_string(), 4),
                            _ => ("???".to_string(), 0),
                        };

                        self.placements[player.id.clone()] = place;

                        if let Some(points_status) = vehicle.player_status_text.points {
                            ui_text
                                .get_mut(points_status)
                                .unwrap()
                                .text = text_out;
                        }
                    }
                } else {
                    if let Some(points_status) = vehicle.player_status_text.points {
                        ui_text
                            .get_mut(points_status)
                            .unwrap()
                            .text = format!("{:.0}", displayed_player_score);
                    }
                }

                //Non-time based game-end condition
                if game_mode_setup.game_end_condition == GameEndCondition::First {
                    if self.winners.len() > 0 || self.losers.len() > 0 {
                        game_score.game_ended = true;
                    }
                }
                else if game_mode_setup.game_end_condition == GameEndCondition::AllButOne {
                    //If all but one player has won (or all but one player has lost)
                    if self.winners.len() >= game_mode_setup.max_players - 1 || 
                            self.losers.len() >= game_mode_setup.max_players - 1 {
                        game_score.game_ended = true;
                    }
                }
                else if game_mode_setup.game_end_condition == GameEndCondition::All {
                    if self.winners.len() == game_mode_setup.max_players || 
                            self.losers.len() == game_mode_setup.max_players {
                        game_score.game_ended = true;
                    }
                }
            }
            else {
                //handle timed games here, player with most points should be displayed as 1st, etc...
                game_score.game_ended = true;
            }
        }

        if game_score.game_ended {
            //Resolve all other placements that are still 0 value
            let mut index_placement_score: Vec<(usize, i32, i32)> = Vec::new();

            for (player_index, score) in self.scores.iter().enumerate() {
                let placement = self.placements[player_index];

                index_placement_score.push((player_index, placement, *score));
            }

            log::info!("{:?}", index_placement_score);

            index_placement_score.sort_by_key(|a| a.1); //first sort by current placement
            index_placement_score.sort_by_key(|a| -a.2); //then sort by score

            log::info!("{:?}", index_placement_score);


            let mut index_final_placement_score: Vec<(usize, i32, i32)> = Vec::new();

            for (new_placement, (player_index, placement, score)) in index_placement_score.iter().enumerate() {
                index_final_placement_score.push((*player_index, (new_placement as i32)+1, *score));
            }

            log::info!("{:?}", index_final_placement_score);
            

            game_score.placements = index_final_placement_score;
        }
    }
}
