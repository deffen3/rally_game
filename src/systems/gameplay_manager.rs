use amethyst::{
    ecs::{System, prelude::*},
    input::{InputHandler, StringBindings},
    derive::SystemDesc,
};

use crate::rally::{CurrentState, UserAction, Game};

#[derive(SystemDesc, Default)]
pub struct GameplayManagerSystem;

impl<'s> System<'s> for GameplayManagerSystem {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        Write<'s, Game>,
    );

    fn run(&mut self, (input, mut game): Self::SystemData) {
        match game.current_state {
            CurrentState::Gameplay => {
                let open_menu = input
                    .action_is_down("open_menu")
                    .unwrap_or(false);

                // Toggle the `open_menu` variable to signal the state to
                // transition.
                if open_menu {
                    game.user_action = Some(UserAction::OpenMenu);
                }
            }
            // do nothing for other states.
            _ => {}
        }
    }
}