use super::helper::{
    distance_to_next_opponent_head, has_neighbour_head, has_wall, move_by_direction,
};
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
    .collect::<Vec<_>>();
    directions.sort_by_cached_key(|d| rank_direction(*d, state));
    debug!("Directions: {:?}", directions);
    if directions.is_empty() {
        None
    } else {
        Some(Command::Move(directions[0].clone()))
    }
}

fn rank_direction(d: &MoveDirection, state: &State) -> impl Ord {
    let next_position = move_by_direction(&state.my_position, &d, &state.game_size);
    let current_space = explore_empty_space(state, next_position.clone());
    (
        has_neighbour_head(&next_position, state),
        OrderedFloat(calculate_best_empty_space_after_step(
            state,
            &move_by_direction(&state.my_position, &d, &state.game_size),
        )),
        !(has_wall(&next_position, state) && current_space.next_snake_head_distance > 4),
        OrderedFloat(evaluate_direction(&d, state)),
    )
}

fn calculate_best_empty_space_after_step(game_state: &State, step_to: &Position) -> f32 {
    let mut new_state = game_state.clone();
    new_state.field_occupation[step_to.as_dim()].replace(game_state.my_id);
    new_state
        .player_heads
        .insert(new_state.my_id, step_to.clone());
    new_state.my_position = step_to.clone();

    let my_min_space = [
        MoveDirection::Up,
        MoveDirection::Down,
        MoveDirection::Left,
        MoveDirection::Right,
    ]
    .iter()
    .filter(|d| {
        !new_state.is_occupied(move_by_direction(
            &new_state.my_position,
            &d,
            &new_state.game_size,
        ))
    })
    .map(|d| {
        OrderedFloat(evaluate_empty_space(&explore_empty_space(
            &new_state,
            move_by_direction(&new_state.my_position, &d, &new_state.game_size),
        )))
    })
    .min()
    .unwrap_or(OrderedFloat(0.0))
    .0;

    my_min_space
}

#[derive(Debug, Default)]
struct EmptySpaceState {
    size: usize,
    num_snake_heads: usize,
    next_snake_head_distance: usize,
    sum_x: u32,
    sum_y: u32,
}

fn explore_empty_space(state: &State, position: Position) -> EmptySpaceState {
    let mut result = EmptySpaceState::default();
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    visited.insert(position.clone());
    queue.push_back((0usize, position.clone()));

    result.next_snake_head_distance = usize::MAX;
    while let Some((dist, p)) = queue.pop_front() {
        if state.player_heads.values().any(|head| *head == p) {
            result.num_snake_heads += 1;
            if state.field_occupation.get(p.as_dim()) != Some(&Some(state.my_id))
                && dist < result.next_snake_head_distance
            {
                result.next_snake_head_distance = dist;
            }
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
                    queue.push_back((dist + 1, next_pos));
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

    distance_to_next_opponent_head(&next_position, state)
        .map(|dist| 1.0 / dist as f32)
        .unwrap_or(0.0)
}
