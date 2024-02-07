/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/client
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use crate::assent::{Assent, AssentCallback, UpdateState};
use crate::seer::{Seer, SeerCallback};
use nimble_steps::Deserialize;

pub trait RectifyCallback {
    fn on_copy_from_authoritative(&mut self);
}

pub struct Rectify<AC, SC, StepT>
where
    StepT: Deserialize,
    AC: AssentCallback<StepT>,
    SC: SeerCallback<StepT>,
{
    assent: Assent<AC, StepT>,
    seer: Seer<SC, StepT>,
}

impl<AC, SC, StepT> Default for Rectify<AC, SC, StepT>
where
    StepT: Deserialize,
    AC: AssentCallback<StepT>,
    SC: SeerCallback<StepT>,
 {
    fn default() -> Self {
        Self::new()
    }
}

impl<AC, SC, StepT> Rectify<AC, SC, StepT>
where
    StepT: Deserialize,
    AC: AssentCallback<StepT>,
    SC: SeerCallback<StepT>,
{
    pub fn new() -> Self {
        let assent = Assent::new();
        let seer = Seer::new();

        Self { assent, seer }
    }

    pub fn update(
        &mut self,
        mut callback: impl RectifyCallback,
        mut ac_callback: AC,
        mut sc_callback: SC,
    ) {
        let consumed_all_knowledge = self.assent.update(&mut ac_callback);
        if consumed_all_knowledge == UpdateState::ConsumedAllKnowledge {
            callback.on_copy_from_authoritative();
        }

        self.seer.update(&mut sc_callback);
    }
}

#[cfg(test)]
mod tests {
    use nimble_steps::{Deserialize, ParticipantSteps, Step};

    use crate::{assent::AssentCallback, seer::SeerCallback};

    pub struct TestGame {
        pub position_x: i32,
    }

    pub struct TestCallback {
        pub authoritative_game: TestGame,
        pub predicted_game: TestGame,
    }

    pub enum TestGameStep {
        MoveLeft,
        MoveRight,
    }

    impl Deserialize for TestGameStep {
        fn deserialize(bytes: &[u8]) -> Self {
            match bytes[0] {
                0 => TestGameStep::MoveRight,
                _ => TestGameStep::MoveLeft,
            }
        }
    }

    impl RectifyCallback for TestCallback {
        fn on_copy_from_authoritative(&mut self) {
            println!("on_copy_from_authoritative");
        }
    }

    impl AssentCallback<TestGameStep> for TestCallback {
        fn on_tick(&mut self, steps: &ParticipantSteps<TestGameStep>) {
            for step in steps.steps.iter() {
                match step.step {
                    Step::Custom(TestGameStep::MoveLeft) => self.authoritative_game.position_x -= 1,
                    Step::Custom(TestGameStep::MoveRight) => {
                        self.authoritative_game.position_x += 1
                    }
                    Step::Forced => todo!(),
                    Step::WaitingForReconnect => todo!(),
                }
            }
        }
    }

    impl SeerCallback<TestGameStep> for TestCallback {
        fn on_tick(&mut self, step: &ParticipantSteps<TestGameStep>) {
            let first = step.steps.first().unwrap();
            match &first.step {
                Step::Custom(game_step) => match game_step {
                    TestGameStep::MoveLeft => {
                        self.predicted_game.position_x -= 1;
                    }
                    TestGameStep::MoveRight => {
                        self.predicted_game.position_x += 1;
                    }
                },
                _ => {}
            }
        }

        fn on_copy_from_authoritative(&mut self) {
            println!("on_copy_from_authoritative");
        }
    }

    #[test]
    fn it_works() {
        let authoritative_game = TestGame { position_x: -44 };
        let predicted_game = TestGame { position_x: -44 };
        let callbacks = TestCallback {
            authoritative_game,
            predicted_game,
        };
        let mut rectify = Rectify::new();
        rectify.update(callbacks, authoritative_game, predicted_game);
        assert_eq!(authoritative_game.position_x, -43);
    }
}
