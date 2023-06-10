use super::helper::{
    distance_to_next_opponent_head, has_wall, move_by_direction, point_to_point_distance,
};
use super::State;
use crate::{AlgorithmConfig, Command, MoveDirection, Position};
use log::debug;
use ordered_float::OrderedFloat;
use rand::rngs::ThreadRng;
use rand::Rng;

pub fn decide_action(
    state: &mut State,
    rng: &mut ThreadRng,
    _config: &AlgorithmConfig,
) -> Option<Command<'static>> {
    let opponent_rooms = evaluate_opponents_rooms(state);

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
            OrderedFloat(evaluate_direction(&d, &r, state, rng, &opponent_rooms)),
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
        }
    }
    return result;
}

fn evaluate_empty_space(state: &EmptySpaceState) -> f32 {
    if state.num_snake_heads == 0 {
        0f32
    } else {
        -(state.size as f32) / (state.num_snake_heads as f32).sqrt()
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

fn evaluate_direction(
    d: &MoveDirection,
    empty_space: &EmptySpaceState,
    state: &State,
    rng: &mut ThreadRng,
    opponent_rooms: &Vec<f32>,
) -> f32 {
    let next_position = move_by_direction(&state.my_position, d, &state.game_size);
    let use_compact_mode = empty_space.num_snake_heads == 1
        || evaluate_empty_space(empty_space)
            <= 0.8
                * opponent_rooms
                    .iter()
                    .map(|f| OrderedFloat(*f))
                    .max()
                    .unwrap_or(OrderedFloat(0.0))
                    .0;

    let mut result =
        1.0 / distance_to_next_opponent_head(&next_position, state).unwrap_or(u32::MAX) as f32;
    if !use_compact_mode {
        result += rng.gen_range(0.0..0.2);
    } else {
        if has_wall(&next_position, &state) {
            result -= 0.03;
        }
    }
    return result;
}
