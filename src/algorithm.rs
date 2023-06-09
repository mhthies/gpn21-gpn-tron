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
use ordered_float::OrderedFloat;

#[derive(Default)]
pub struct State {
    my_id: PlayerId,
    my_position: Position,
    /// PlayerId per field
    field_occupation: ndarray::Array2<Option<PlayerId>>,
    player_heads: HashMap<PlayerId, Position>,
    game_size: Position,
}

impl State {
    pub fn update_from_answer(&mut self, answer: &Answer) {
        match answer {
            Answer::Pos(p, position) => {
                if *p == self.my_id {
                    self.my_position = position.clone();
                }
                self.player_heads.insert(p.clone(), position.clone());
                self.field_occupation[position.as_dim()] = Some(p.clone());
            },
            Answer::Game(size, my_id) => {
                self.my_id = my_id.clone();
                self.field_occupation = ndarray::Array2::from_elem(size.as_dim(), None);
                self.game_size = size.clone();
                self.player_heads.clear();
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
    let mut directions = [MoveDirection::Up, MoveDirection::Down, MoveDirection::Left, MoveDirection::Right]
        .iter()
        .map(|d| (explore_empty_space(&state, move_by_direction(&state.my_position, &d, &state.game_size)), d))
        .collect::<Vec<_>>();
    directions.sort_by_key(|(r, d)| (OrderedFloat(evaluate_empty_space(&r))));
    debug!("Directions: {:?}", directions);
    if directions.is_empty() {
        None
    } else {
        Some(Command::Move(directions[0].1.clone()))
    }
}

#[derive(Debug)]
struct EmptySpaceState {
    size: usize,
    num_snake_heads: usize,
}

fn explore_empty_space(state: &State, position: Position) -> EmptySpaceState {
    let mut result = EmptySpaceState{ size: 0, num_snake_heads: 0};
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    visited.insert(position.clone());
    queue.push_back(position.clone());

    while let Some(p) = queue.pop_front() {
        if state.player_heads.values().any(|head| *head == p) {
            result.num_snake_heads += 1;
        }
        if !state.is_occupied(p.clone()) {
            result.size += 1;
            for direction in [MoveDirection::Up, MoveDirection::Down, MoveDirection::Left, MoveDirection::Right] {
                let next_pos = move_by_direction(&p, &direction, &state.game_size);
                if !visited.contains(&next_pos) {
                    visited.insert(next_pos.clone());
                    queue.push_back(next_pos);
                }
            }
        }
    }
    return result;
}

fn evaluate_empty_space(state: &EmptySpaceState) -> f32 {
    if state.num_snake_heads == 0 {0f32} else {- (state.size as f32) / (state.num_snake_heads as f32).sqrt() }
}


