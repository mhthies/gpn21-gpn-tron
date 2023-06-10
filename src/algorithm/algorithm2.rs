use super::helper::{has_neighbour_head, has_wall, move_by_direction};
use super::State;
use crate::{AlgorithmConfig, Command, MoveDirection, Position};
use log::debug;
use ordered_float::OrderedFloat;
use rand::rngs::ThreadRng;

pub fn decide_action(
    state: &mut State,
    _rng: &mut ThreadRng,
    _config: &AlgorithmConfig,
) -> Option<Command<'static>> {
    let tainted_fields = taint_fields_near_heads(state);
    let mut directions = [
        MoveDirection::Up,
        MoveDirection::Down,
        MoveDirection::Left,
        MoveDirection::Right,
    ]
    .iter()
    .filter(|d| !state.is_occupied(move_by_direction(&state.my_position, &d, &state.game_size)))
    .map(|d| {
        (
            evaluate_direction_weighted(
                &state,
                &move_by_direction(&state.my_position, &d, &state.game_size),
                &tainted_fields,
            ),
            d,
        )
    })
    .collect::<Vec<_>>();
    directions.sort_by_key(|(r, _d)| OrderedFloat(*r));
    debug!("Directions: {:?}", directions);
    if directions.is_empty() {
        None
    } else {
        Some(Command::Move(directions[0].1.clone()))
    }
}

const MAX_FIELD_DISTANCE_SCALING: f32 = 1.0;
const MIN_FIELD_DISTANCE_SCALING: f32 = 0.9;
const FACTOR_WALL: f32 = 1.05;
const FACTOR_HEAD: f32 = 0.85;
const DISTANCE_EXPONENT: f32 = 0.6;

fn evaluate_direction_weighted(
    state: &State,
    position: &Position,
    tainted_fields: &ndarray::Array2<f32>,
) -> f32 {
    let mut result = 0.0;
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    visited.insert(position.clone());
    queue.push_back((0usize, position.clone()));

    while let Some((dist, p)) = queue.pop_front() {
        if !state.is_occupied(p.clone()) {
            let scaling = (1.0 / ((dist + 1) as f32).powf(DISTANCE_EXPONENT))
                * (MAX_FIELD_DISTANCE_SCALING - MIN_FIELD_DISTANCE_SCALING)
                + MIN_FIELD_DISTANCE_SCALING;
            result += tainted_fields[p.as_dim()] * scaling;
            for direction in [
                MoveDirection::Up,
                MoveDirection::Down,
                MoveDirection::Left,
                MoveDirection::Right,
            ] {
                let next_pos = move_by_direction(&p, &direction, &state.game_size);
                if !visited.contains(&next_pos) {
                    visited.insert(next_pos.clone());
                    queue.push_back((dist + 1, next_pos));
                }
            }
        }
    }
    return -result
        * if has_wall(position, state) {
            FACTOR_WALL
        } else if has_neighbour_head(position, state) {
            FACTOR_HEAD
        } else {
            1.0
        };
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
            for direction in [
                MoveDirection::Up,
                MoveDirection::Down,
                MoveDirection::Left,
                MoveDirection::Right,
            ] {
                let next_pos = move_by_direction(&p, &direction, &state.game_size);
                if !visited.contains(&next_pos) && !state.is_occupied(next_pos.clone()) {
                    visited.insert(next_pos.clone());
                    queue.push_back((dist + 1, next_pos));
                }
            }
        }
    }

    return result;
}
