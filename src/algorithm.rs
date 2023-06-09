use core::option::Option;
use core::option::Option::{None, Some};
use crate::{AlgorithmConfig, helper, MoveDirection, Position};
use crate::client::{Answer, Command, PlayerId};
use rand::rngs::ThreadRng;
use std::collections::{HashMap, VecDeque, HashSet};
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
                self.player_heads.remove(p);
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
    if config.use_weighted_space_exploration {
        let tainted_fields = taint_fields_near_heads(state);
        let mut directions = [MoveDirection::Up, MoveDirection::Down, MoveDirection::Left, MoveDirection::Right]
            .iter()
            .filter(|d| !state.is_occupied(move_by_direction(&state.my_position, &d, &state.game_size)))
            .map(|d| (evaluate_direction_weighted(&state, &move_by_direction(&state.my_position, &d, &state.game_size), &tainted_fields), d))
            .collect::<Vec<_>>();
        directions.sort_by_key(|(r, d)| OrderedFloat(*r));
        debug!("Directions: {:?}", directions);
        if directions.is_empty() {
            None
        } else {
            Some(Command::Move(directions[0].1.clone()))
        }
    } else {
        let mut directions = [MoveDirection::Up, MoveDirection::Down, MoveDirection::Left, MoveDirection::Right]
            .iter()
            .filter(|d| !state.is_occupied(move_by_direction(&state.my_position, &d, &state.game_size)))
            .map(|d| (explore_empty_space(&state, move_by_direction(&state.my_position, &d, &state.game_size)), d))
            .collect::<Vec<_>>();
        directions.sort_by_key(|(r, d)| (OrderedFloat(evaluate_empty_space(&r)), OrderedFloat(evaluate_direction(&d, state))));
        debug!("Directions: {:?}", directions);
        if directions.is_empty() {
            None
        } else {
            Some(Command::Move(directions[0].1.clone()))
        }
    }
}

#[derive(Debug, Default)]
struct EmptySpaceState {
    size: usize,
    num_snake_heads: usize,
    sum_x: u32,
    sum_y: u32,
}

impl EmptySpaceState {
    fn mass_center(&self) -> (f32, f32) {
        (
            (self.sum_x as f32) / (self.size as f32),
            (self.sum_y as f32) / (self.size as f32),
        )
    }
}

fn explore_empty_space(state: &State, position: Position) -> EmptySpaceState {
    let mut result = EmptySpaceState::default();
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
            result.sum_x += p.x;
            result.sum_y += p.y;
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

fn evaluate_direction(d: &MoveDirection, state: &State) -> f32 {
    let next_position = move_by_direction(&state.my_position, d, &state.game_size);

    let min_player_distance: f32 = state.player_heads.iter()
        .filter(|(p, _pos)| **p != state.my_id)
        .map(|(_p, pos)| OrderedFloat(point_to_point_distance(&next_position, &pos, &state.game_size)))
        .min()
        .unwrap().0;

    1.0 / min_player_distance - if has_wall(&next_position, state) { 0.03 } else {0.0}
}

fn point_to_float_point_distance(p: &Position, x2: f32, y2: f32, game_size: &Position) -> f32 {
    return [
        OrderedFloat(((p.x as f32 - x2).powi(2) + (p.y as f32 - y2).powi(2)).sqrt()),
        OrderedFloat(((p.x as f32 - x2).powi(2) + ((p.y + game_size.y) as f32 - y2).powi(2)).sqrt()),
        OrderedFloat((((p.x + game_size.x) as f32 - x2).powi(2) + (p.y as f32 - y2).powi(2)).sqrt()),
        OrderedFloat((((p.x + game_size.x) as f32 - x2).powi(2) + ((p.y + game_size.y) as f32 - y2).powi(2)).sqrt()),
    ].iter().min().unwrap().0
}

fn point_to_point_distance(p1: &Position, p2: &Position, game_size: &Position) -> f32 {
    point_to_float_point_distance(p1, p2.x as f32, p2.y as f32, game_size)
}

fn has_wall(pos: &Position, game_state: &State) -> bool {
    [MoveDirection::Up, MoveDirection::Down, MoveDirection::Left, MoveDirection::Right].iter()
        .map(|d|move_by_direction(pos, d, &game_state.game_size))
        .filter(|p| !game_state.player_heads.values().any(|head| *p == *head))
        .map(|p| game_state.is_occupied(p))
        .any(|b| b)
}

fn has_head(pos: &Position, game_state: &State) -> bool {
    [MoveDirection::Up, MoveDirection::Down, MoveDirection::Left, MoveDirection::Right].iter()
        .map(|d|move_by_direction(pos, d, &game_state.game_size))
        .map(|p| {
            game_state.player_heads.iter()
                .filter(|(player, _head)| **player != game_state.my_id)
                .any(|(_player, head)| p == *head)})
        .any(|b| b)
}


// New algorithm

const MAX_FIELD_DISTANCE_SCALING: f32 = 1.0;
const MIN_FIELD_DISTANCE_SCALING: f32 = 0.9;
const FACTOR_WALL: f32 = 1.05;
const FACTOR_HEAD: f32 = 0.85;
const DISTANCE_EXPONENT: f32 = 0.6;

fn evaluate_direction_weighted(state: &State, position: &Position, tainted_fields: &ndarray::Array2<f32>) -> f32 {

    let mut result = 0.0;
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    visited.insert(position.clone());
    queue.push_back((0usize, position.clone()));

    while let Some((dist, p)) = queue.pop_front() {
        if !state.is_occupied(p.clone()) {
            let scaling = (1.0 / ((dist+1) as f32).powf(DISTANCE_EXPONENT)) * (MAX_FIELD_DISTANCE_SCALING - MIN_FIELD_DISTANCE_SCALING) + MIN_FIELD_DISTANCE_SCALING;
            result += tainted_fields[p.as_dim()] * scaling;
            for direction in [MoveDirection::Up, MoveDirection::Down, MoveDirection::Left, MoveDirection::Right] {
                let next_pos = move_by_direction(&p, &direction, &state.game_size);
                if !visited.contains(&next_pos) {
                    visited.insert(next_pos.clone());
                    queue.push_back((dist+1, next_pos));
                }
            }
        }
    }
    return - result * if has_wall(position, state) { FACTOR_WALL } else if has_head(position, state) { FACTOR_HEAD } else  { 1.0 };
}

const MAX_FIELD_SCORE: f32 = 1.0;
const MIN_FIELD_SCORE: f32 = 0.4;
const FIELD_SCORE_ALPHA: f32 = 0.6;

// IDEA: field score = 1.0 * (1 - (MIN_FIELD_SCORE ^ (alpha * distance_1))) * (1 - (MIN_FIELD_SCORE ^ (alpha * distance_2))) ...
fn taint_fields_near_heads(state: &State) -> ndarray::Array2<f32> {
    let mut result = ndarray::Array2::from_elem(state.game_size.as_dim(), MAX_FIELD_SCORE);

    for (player, head) in state.player_heads.iter() {
        if *player == state.my_id {
            continue;
        }

        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        visited.insert(head.clone());
        queue.push_back((-1i32, head.clone()));

        while let Some((dist, p)) = queue.pop_front() {
            result[p.as_dim()] *= 1.0 - MIN_FIELD_SCORE.powf(dist as f32 * FIELD_SCORE_ALPHA);
            for direction in [MoveDirection::Up, MoveDirection::Down, MoveDirection::Left, MoveDirection::Right] {
                let next_pos = move_by_direction(&p, &direction, &state.game_size);
                if !visited.contains(&next_pos) && !state.is_occupied(next_pos.clone()){
                    visited.insert(next_pos.clone());
                    queue.push_back((dist+1, next_pos));
                }
            }
        }
    }

    return result;

}
