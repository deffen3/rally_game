use amethyst::{
    core::transform::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, ReadExpect, System, SystemData, Write, WriteStorage, ReadStorage},
    ui::UiText,
};

use crate::rally::{ScoreBoard, ScoreText, Vehicle};


#[derive(SystemDesc)]
pub struct VehicleStatusSystem;

impl<'s> System<'s> for VehicleStatusSystem {
    type SystemData = (
        ReadStorage<'s, Vehicle>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, UiText>,
        Write<'s, ScoreBoard>,
        ReadExpect<'s, ScoreText>,
    );

    fn run(&mut self, (
        vehicles,
        _transforms,
        mut ui_text,
        mut scores,
        score_text,
    ): Self::SystemData)  {
        for _vehicle in vehicles.join() {
            
            // scores.score_right = (scores.score_right + 1)
            //     .min(10);

            // if let Some(text) = ui_text.get_mut(score_text.p2_score) {
            //     text.text = scores.score_right.to_string();
            // }

            // scores.score_left = (scores.score_left + 1)
            //     .min(10);

            // if let Some(text) = ui_text.get_mut(score_text.p1_score) {
            //     text.text = scores.score_left.to_string();
            // }
        }
    }
}