use super::helper::{has_wall, move_by_direction, point_to_point_distance};
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
            OrderedFloat(evaluate_direction(&d, state)),
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

fn evaluate_direction(d: &MoveDirection, state: &State) -> f32 {
    let next_position = move_by_direction(&state.my_position, d, &state.game_size);

    let min_player_distance: f32 = state
        .player_heads
        .iter()
        .filter(|(p, _pos)| **p != state.my_id)
        .map(|(_p, pos)| {
            OrderedFloat(point_to_point_distance(
                &next_position,
                &pos,
                &state.game_size,
            ))
        })
        .min()
        .unwrap()
        .0;

    1.0 / min_player_distance
        - if has_wall(&next_position, state) {
            0.03
        } else {
            0.0
        }
}
