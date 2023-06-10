use std::collections::HashSet;

use super::helper::{
    distance_to_next_opponent_head, has_wall, move_by_direction,
};
use super::State;
use crate::client::PlayerId;
use crate::{AlgorithmConfig, Command, MoveDirection, Position};
use log::{debug, info};
use ordered_float::OrderedFloat;
use rand::rngs::ThreadRng;
use rand::Rng;

pub fn decide_action(
    state: &mut State,
    rng: &mut ThreadRng,
    _config: &AlgorithmConfig,
) -> Option<Command<'static>> {
    let opponent_rooms = evaluate_opponents_rooms(state);
    let tainted_fields = taint_fields_near_heads(state);
    debug!("Opponent rooms: {:?}", opponent_rooms);

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
            explore_empty_space(
                &state,
                move_by_direction(&state.my_position, &d, &state.game_size),
            ),
            d,
        )
    })
    .collect::<Vec<_>>();
    directions.sort_by_key(|(r, d)| {
        (
            OrderedFloat(evaluate_empty_space(&r)),
            OrderedFloat(evaluate_direction(&d, &r, state, rng, &opponent_rooms, &tainted_fields)),
        )
    });
    debug!("Directions: {:?}", directions);
    if directions.is_empty() {
        None
    } else {
        Some(Command::Move(directions[0].1.clone()))
    }
}

#[derive(Debug, Default)]
struct EmptySpaceState {
    size: usize,
    num_snake_heads: usize,
    wall_players: HashSet<PlayerId>,
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
        }
        if !state.is_occupied(p.clone()) || p == position {
            for direction in [
                MoveDirection::Up,
                MoveDirection::Down,
                MoveDirection::Left,
                MoveDirection::Right,
            ] {
                let next_pos = move_by_direction(&p, &direction, &state.game_size);
                if !visited.contains(&next_pos) {
                    visited.insert(next_pos.clone());
                    queue.push_back(next_pos);
                }
            }
        } else {
            result.wall_players.insert(state.field_occupation[p.as_dim()].unwrap());
        }
    }
    return result;
}

fn evaluate_empty_space(state: &EmptySpaceState) -> f32 {
    if state.num_snake_heads == 0 {
        0f32
    } else {
        -(state.size as f32) / (state.num_snake_heads as f32).powf(0.5) * (state.wall_players.len() as f32).powf(0.1)
    }
}

fn evaluate_opponents_rooms(state: &State) -> Vec<f32> {
    state
        .player_heads
        .iter()
        .filter(|(player, _head)| **player != state.my_id)
        .map(|(_player, head)| explore_empty_space(state, head.clone()))
        .map(|space| evaluate_empty_space(&space))
        .collect()
}

type FieldTaint = ndarray::Array2<f32>;

fn evaluate_direction(
    d: &MoveDirection,
    empty_space: &EmptySpaceState,
    state: &State,
    _rng: &mut ThreadRng,
    opponent_rooms: &Vec<f32>,
    tainted_fields: &FieldTaint,
) -> f32 {
    let next_position = move_by_direction(&state.my_position, d, &state.game_size);
    let use_compact_mode = empty_space.num_snake_heads <= 2
        || evaluate_empty_space(empty_space)
            > 0.8 * opponent_rooms
                    .iter()
                    .map(|f| OrderedFloat(*f))
                    .min()
                    .unwrap_or(OrderedFloat(0.0))
                    .0;
    info!("{}using compact mode.", if use_compact_mode {""} else {"not "});

    if use_compact_mode {
        let mut result =
            1.0 / distance_to_next_opponent_head(&next_position, state).unwrap_or(u32::MAX) as f32;
        if has_wall(&next_position, &state) {
            result -= 0.03;
        }
        result
    } else {
        - evaluate_direction_weighted(state, &next_position, tainted_fields)
    }
}


fn evaluate_direction_weighted(
    state: &State,
    position: &Position,
    tainted_fields: &FieldTaint,
) -> f32 {
    let mut result = 0.0;
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    visited.insert(position.clone());
    queue.push_back((1.0, position.clone()));

    while let Some((scale, p)) = queue.pop_front() {
        if !state.is_occupied(p.clone()) {
            let score = tainted_fields[p.as_dim()];
            result += scale * score;
            for direction in [
                MoveDirection::Up,
                MoveDirection::Down,
                MoveDirection::Left,
                MoveDirection::Right,
            ] {
                let next_pos = move_by_direction(&p, &direction, &state.game_size);
                if !visited.contains(&next_pos) {
                    visited.insert(next_pos.clone());
                    queue.push_back((score * scale, next_pos));
                }
            }
        }
    }
    return result;
}

const MAX_FIELD_SCORE: f32 = 0.95;
const MIN_FIELD_SCORE: f32 = 0.4;
const FIELD_SCORE_ALPHA: f32 = 0.8;

// IDEA: field score = 1.0 * (1 - (MIN_FIELD_SCORE ^ (alpha * distance_1))) * (1 - (MIN_FIELD_SCORE ^ (alpha * distance_2))) ...
fn taint_fields_near_heads(state: &State) -> FieldTaint {
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

