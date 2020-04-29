use amethyst::ecs::{Join, ReadStorage, System, SystemData, WriteStorage, World};
use amethyst::{derive::SystemDesc, ui::UiText};

use crate::components::{Player, Vehicle};
use crate::rally::KILLS_TO_WIN;

#[derive(SystemDesc, Default)]
pub struct VehicleStatusSystem {
    pub winners: Vec<usize>,
}

impl<'s> System<'s> for VehicleStatusSystem {
    type SystemData = (

        ReadStorage<'s, Player>,
        ReadStorage<'s, Vehicle>,
        WriteStorage<'s, UiText>,
    );

    fn setup(&mut self, _world: &mut World) {
        self.winners = vec![];
    }

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
                .text = format!("{:.0}", vehicle.armor.value.ceil());

            ui_text
                .get_mut(vehicle.player_status_text.health)
                .unwrap()
                .text = format!("{:.0}", vehicle.health.value.ceil());


            if player.kills >= KILLS_TO_WIN {
                if self.winners.contains(&player.id) {
                    //pass
                }
                else {
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
                    .text = format!("{:.0}", player.kills);
            }
        }
    }
}
