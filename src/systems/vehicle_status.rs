use amethyst::ecs::{Join, ReadStorage, System, SystemData, WriteStorage};
use amethyst::{derive::SystemDesc, ui::UiText};

use crate::components::{Player, Vehicle};
use crate::rally::KILLS_TO_WIN;

#[derive(SystemDesc)]
pub struct VehicleStatusSystem;

impl<'s> System<'s> for VehicleStatusSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        WriteStorage<'s, UiText>,
    );

    fn run(&mut self, (players, vehicles, mut ui_text): Self::SystemData) {
        //for (player, vehicle) in (players, vehicles).join() {
        for (player, vehicle) in (&players, &vehicles).join() {
            ui_text
                .get_mut(vehicle.player_status_text.shield)
                .unwrap()
                .text = format!("{:.0}", vehicle.shield.value.ceil());
                
            ui_text
                .get_mut(vehicle.player_status_text.armor)
                .unwrap()
                .text = format!("{:.0}", vehicle.armor.ceil());

            ui_text
                .get_mut(vehicle.player_status_text.health)
                .unwrap()
                .text = format!("{:.0}", vehicle.health.ceil());


            if player.kills >= KILLS_TO_WIN {
                ui_text
                    .get_mut(vehicle.player_status_text.kills)
                    .unwrap()
                    .text = "WIN!".to_string();
            }
            else {
                ui_text
                    .get_mut(vehicle.player_status_text.kills)
                    .unwrap()
                    .text = format!("{:.0}", player.kills);
            }
        }
    }
}
