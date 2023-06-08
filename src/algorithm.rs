use core::cmp::Ordering::Equal;
use core::option::Option;
use core::option::Option::{None, Some};
use crate::{AlgorithmConfig, helper, MoveDirection, Position};
use crate::client::{Answer, Command};
use rand::rngs::ThreadRng;
use std::collections::{HashMap, VecDeque};
use std::collections::HashSet;
use log::{debug, info, warn};
use crate::helper::{direction_from_move, distance_from_line, move_by_direction};

#[derive(Default)]
pub struct State {
    // TODO
}

impl State {
    pub fn update_from_answer(&mut self, answer: &Answer) {
        match answer {
            Answer::Pos(p, w) => {
                // TODO
            },
            Answer::Game(size, goal) => {
                // TODO
            },
            _ => {},
        }
    }

    fn reset(&mut self) {
        // TODO
    }
}

pub fn decide_action(state: &mut State, rng: &mut ThreadRng, config: &AlgorithmConfig) -> Option<Command<'static>> {
    // TODO
    Some(Command::Move(MoveDirection::Up))
}
