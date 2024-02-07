/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/client
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use nimble_steps::{Deserialize, ParticipantSteps, Step, Steps};
use std::marker::PhantomData;

pub trait AssentCallback<StepT: Deserialize> {
    fn on_pre_ticks(&mut self) {}

    fn on_tick(&mut self, step: &ParticipantSteps<StepT>);
}

#[derive(Debug, PartialEq)]
pub enum UpdateState {
    ConsumedAllKnowledge,
    DidNotConsumeAllKnowledge,
}

// Define the Assent struct
pub struct Assent<C, StepT>
where
    StepT: Deserialize,
    C: AssentCallback<StepT>,
{
    phantom: PhantomData<C>,
    steps: Steps<StepT>,
}

impl<C, StepT> Default for Assent<C, StepT>
where
    StepT: Deserialize,
    C: AssentCallback<StepT>,
{
    fn default() -> Self {
        Assent::new()
    }
}

impl<C, StepT> Assent<C, StepT>
where
    StepT: Deserialize,
    C: AssentCallback<StepT>,
{
    pub fn new() -> Self {
        Assent {
            phantom: PhantomData {},
            steps: Steps::new(),
        }
    }

    pub fn push(&mut self, steps: ParticipantSteps<StepT>) {
        self.steps.push(steps);
    }

    pub fn update(&mut self, callback: &mut C) -> UpdateState {
        let bytes = [0u8; 1];
        let step = StepT::deserialize(&bytes);
        let mut steps = ParticipantSteps::new();
        steps.push(42, Step::Custom(step));
        callback.on_tick(&steps);

        UpdateState::ConsumedAllKnowledge
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nimble_steps::Step;

    pub struct TestGame {
        pub position_x: i32,
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

    impl AssentCallback<TestGameStep> for TestGame {
        fn on_tick(&mut self, steps: &ParticipantSteps<TestGameStep>) {
            for step in steps.steps.iter() {
                match step.step {
                    Step::Custom(TestGameStep::MoveLeft) => self.position_x -= 1,
                    Step::Custom(TestGameStep::MoveRight) => self.position_x += 1,
                    Step::Forced => todo!(),
                    Step::WaitingForReconnect => todo!(),
                }
            }
        }
    }

    #[test]
    fn test_assent() {
        let mut game = TestGame { position_x: -44 };
        let mut assent: Assent<TestGame, TestGameStep> = Assent::new();
        let mut steps = ParticipantSteps::new();
        let step = TestGameStep::MoveLeft;
        let custom_step = Step::Custom(step);
        steps.push(42, custom_step);
        assent.push(steps);
        assent.update(&mut game);
        assert_eq!(game.position_x, -43);
    }
}
