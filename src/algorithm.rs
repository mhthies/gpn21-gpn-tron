use core::cmp::Ordering::Equal;
use core::option::Option;
use core::option::Option::{None, Some};
use crate::{AlgorithmConfig, helper, MoveDirection, Position};
use crate::client::{Answer, Command, PlayerId};
use rand::rngs::ThreadRng;
use std::collections::{HashMap, VecDeque};
use std::collections::HashSet;
use log::{debug, info, warn};
use crate::helper::{distance_from_line, move_by_direction};

#[derive(Default)]
pub struct State {
    my_id: PlayerId,
    my_position: Position,
    /// PlayerId per field
    field_occupation: ndarray::Array2<Option<PlayerId>>,
    game_size: Position,
}

impl State {
    pub fn update_from_answer(&mut self, answer: &Answer) {
        match answer {
            Answer::Pos(p, position) => {
                if *p == self.my_id {
                    self.my_position = position.clone();
                }
                self.field_occupation[position.as_dim()] = Some(p.clone());
            },
            Answer::Game(size, my_id) => {
                self.my_id = my_id.clone();
                self.field_occupation = ndarray::Array2::from_elem(size.as_dim(), None);
                self.game_size = size.clone();
            },
            Answer::Die(p) => {
                for field in self.field_occupation.iter_mut() {
                    if *field == Some(p.clone()) {
                        *field = None;
                    }
                }
            },
            _ => {},
        }
    }

    fn reset(&mut self) {
        self.my_position = Position{x: 0, y: 0};
        self.game_size = Position{x: 0, y: 0};
        self.field_occupation = ndarray::Array2::from_elem((0, 0), None);
    }

    fn is_occupied(&self, p: Position) -> bool {
        self.field_occupation[p.as_dim()].is_some()
    }
}

pub fn decide_action(state: &mut State, rng: &mut ThreadRng, config: &AlgorithmConfig) -> Option<Command<'static>> {
    if state.game_size.x == 0 || state.game_size.y == 0 {
        return None;
    }
    for direction in [MoveDirection::Up, MoveDirection::Down, MoveDirection::Left, MoveDirection::Right] {
        if !state.is_occupied(move_by_direction(&state.my_position, &direction, &state.game_size)) {
            return Some(Command::Move(direction))
        }
    }
    None
}
