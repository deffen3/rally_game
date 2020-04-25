use amethyst::{         
    derive::SystemDesc,
    ui::UiText,
};
use amethyst::ecs::{Join, System, SystemData, WriteStorage, ReadExpect};

use crate::components::{Vehicle, Player};
use crate::entities::{ScoreText};
use crate::rally::{KILLS_TO_WIN};

#[derive(SystemDesc)]
pub struct VehicleStatusSystem;

impl<'s> System<'s> for VehicleStatusSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, UiText>,
        ReadExpect<'s, ScoreText>,
    );

    fn run(&mut self, (
        mut players,
        mut vehicles,
        mut ui_text,
        score_text,
    ): Self::SystemData)  {
        //for (player, vehicle) in (players, vehicles).join() {
        for (player, vehicle) in (&mut players, &mut vehicles).join() {
            
            if player.id == 0 {
                let shield: i32 = vehicle.shield.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p1_shield) {
                    text.text = shield.to_string();
                }

                let armor: i32 = vehicle.armor.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p1_armor) {
                    text.text = armor.to_string();
                }

                let health: i32 = vehicle.health.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p1_health) {
                    text.text = health.to_string();
                }

                let kills: i32 = player.kills as i32;
                if kills == KILLS_TO_WIN {
                    if let Some(text) = ui_text.get_mut(score_text.p1_kills) {
                        text.text = "WIN!".to_string();
                    }
                }
                else {
                    if let Some(text) = ui_text.get_mut(score_text.p1_kills) {
                        text.text = kills.to_string();
                    }
                }
                
                
            }
            if player.id == 1 {
                let shield: i32 = vehicle.shield.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p2_shield) {
                    text.text = shield.to_string();
                }

                let armor: i32 = vehicle.armor.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p2_armor) {
                    text.text = armor.to_string();
                }

                let health: i32 = vehicle.health.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p2_health) {
                    text.text = health.to_string();
                }
            }
            if player.id == 2 {
                let shield: i32 = vehicle.shield.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p3_shield) {
                    text.text = shield.to_string();
                }

                let armor: i32 = vehicle.armor.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p3_armor) {
                    text.text = armor.to_string();
                }

                let health: i32 = vehicle.health.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p3_health) {
                    text.text = health.to_string();
                }
            }
            if player.id == 3 {
                let shield: i32 = vehicle.shield.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p4_shield) {
                    text.text = shield.to_string();
                }

                let armor: i32 = vehicle.armor.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p4_armor) {
                    text.text = armor.to_string();
                }

                let health: i32 = vehicle.health.ceil() as i32;
                if let Some(text) = ui_text.get_mut(score_text.p4_health) {
                    text.text = health.to_string();
                }
            }
        }
    }
}