use amethyst::ecs::{Join, ReadExpect, System, SystemData, WriteStorage};
use amethyst::{derive::SystemDesc, ui::UiText};

use crate::components::{Player, Vehicle};
use crate::entities::ScoreText;
use crate::rally::KILLS_TO_WIN;

#[derive(SystemDesc)]
pub struct VehicleStatusSystem;

impl<'s> System<'s> for VehicleStatusSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, UiText>,
        ReadExpect<'s, ScoreText>,
    );

    fn run(&mut self, (mut players, mut vehicles, mut ui_text, score_text): Self::SystemData) {
        //for (player, vehicle) in (players, vehicles).join() {
        for (player, vehicle) in (&mut players, &mut vehicles).join() {
            ui_text
                .get_mut(vehicle.player_status_text.shield)
                .unwrap()
                .text = format!("{:.0}", vehicle.shield.ceil());
            ui_text
                .get_mut(vehicle.player_status_text.armor)
                .unwrap()
                .text = format!("{:.0}", vehicle.armor.ceil());

            if player.id == 0 {
                let health: i32 = vehicle.health.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p1_health) {
                    text.text = health.to_string();
                }

                let kills: i32 = player.kills as i32;
                if kills >= KILLS_TO_WIN {
                    if let Some(text) = ui_text.get_mut(score_text.p1_kills) {
                        text.text = "WIN!".to_string();
                    }
                } else {
                    if let Some(text) = ui_text.get_mut(score_text.p1_kills) {
                        text.text = kills.to_string();
                    }
                }
            }
            if player.id == 1 {
                let health: i32 = vehicle.health.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p2_health) {
                    text.text = health.to_string();
                }

                let kills: i32 = player.kills as i32;
                if kills >= KILLS_TO_WIN {
                    if let Some(text) = ui_text.get_mut(score_text.p2_kills) {
                        text.text = "WIN!".to_string();
                    }
                } else {
                    if let Some(text) = ui_text.get_mut(score_text.p2_kills) {
                        text.text = kills.to_string();
                    }
                }
            }
            if player.id == 2 {
                let health: i32 = vehicle.health.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p3_health) {
                    text.text = health.to_string();
                }

                let kills: i32 = player.kills as i32;
                if kills >= KILLS_TO_WIN {
                    if let Some(text) = ui_text.get_mut(score_text.p3_kills) {
                        text.text = "WIN!".to_string();
                    }
                } else {
                    if let Some(text) = ui_text.get_mut(score_text.p3_kills) {
                        text.text = kills.to_string();
                    }
                }
            }
            if player.id == 3 {
                let health: i32 = vehicle.health.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p4_health) {
                    text.text = health.to_string();
                }

                let kills: i32 = player.kills as i32;
                if kills >= KILLS_TO_WIN {
                    if let Some(text) = ui_text.get_mut(score_text.p4_kills) {
                        text.text = "WIN!".to_string();
                    }
                } else {
                    if let Some(text) = ui_text.get_mut(score_text.p4_kills) {
                        text.text = kills.to_string();
                    }
                }
            }
        }
    }
}
