use std::collections::HashSet;
use std::thread::scope;

use super::helper::{
    distance_to_next_opponent_head, has_neighbour_head, has_wall, iter_directions,
    move_by_direction,
};
use super::State;
use crate::client::PlayerId;
use crate::{AlgorithmConfig, Command, MoveDirection, Position};
use log::{info, warn};
use ordered_float::OrderedFloat;
use rand::rngs::ThreadRng;
use rand::Rng;

pub fn decide_action(
    state: &mut State,
    rng: &mut ThreadRng,
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
    .map(|d| (d, rank_direction(d, state, rng)))
    .collect::<Vec<_>>();

    directions.sort_by_key(|(_d, rank)| rank.clone());
    if directions.is_empty() {
        warn!("No step possible.");
        None
    } else {
        if directions.len() == 1 {
            info!("Only one step possible.");
        } else if directions[0].1.has_neighbour_head != directions[1].1.has_neighbour_head {
            info!("Avoiding other head");
        } else if directions[0].1.best_empty_space_score_after_step != directions[1].1.best_empty_space_score_after_step {
            info!("Room score different: {:?}: {}, {:?}: {}", directions[0].0, directions[0].1.best_empty_space_score_after_step.0, directions[1].0, directions[1].1.best_empty_space_score_after_step.0);
        } else if directions[0].1.direction_score != directions[1].1.direction_score {
            info!("Better direction: {:?}: {}, {:?}: {}", directions[0].0, directions[0].1.direction_score.0, directions[1].0, directions[1].1.direction_score.0);
        } else {
            info!("Using random direction");
        }
    Some(Command::Move(directions[0].0.clone()))
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
struct DirectionRanking {
    has_neighbour_head: bool,
    best_empty_space_score_after_step: OrderedFloat<f32>,
    direction_score: OrderedFloat<f32>,
    random: i32,
}

fn rank_direction(d: &MoveDirection, state: &State, rng: &mut ThreadRng) -> DirectionRanking {
    let next_position = move_by_direction(&state.my_position, &d, &state.game_size);
    let current_space = explore_empty_space(state, next_position.clone());
    DirectionRanking {
        has_neighbour_head: has_neighbour_head(&next_position, state) && state.player_heads.len() > 2,
        best_empty_space_score_after_step: OrderedFloat(calculate_best_empty_space_after_step(
            state,
            &next_position),
        ),
        direction_score: OrderedFloat(evaluate_direction(&next_position, &current_space, state)),
        random: rng.gen(),
    }
}

fn calculate_best_empty_space_after_step(game_state: &State, step_to: &Position) -> f32 {
    let new_state = game_state.simulate_step(step_to);

    let my_min_space = iter_directions()
        .map(|d| move_by_direction(&new_state.my_position, d, &new_state.game_size))
        .filter(|p| !new_state.is_occupied(p.clone()))
        .map(|p| OrderedFloat(evaluate_empty_space(&explore_empty_space(&new_state, p))))
        .min()
        .unwrap_or(OrderedFloat(0.0))
        .0;

    my_min_space
}

#[derive(Debug, Default)]
struct EmptySpaceState {
    size: usize,
    snake_head_distances: Vec<usize>,
    sum_x: u32,
    bounding_snakes: HashSet<PlayerId>,
    wide_room_score: f32,
    sum_y: u32,
}

fn explore_empty_space(state: &State, position: Position) -> EmptySpaceState {
    let mut result = EmptySpaceState::default();
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    visited.insert(position.clone());
    queue.push_back((0usize, position.clone()));

    while let Some((dist, p)) = queue.pop_front() {
        if state.player_heads.values().any(|head| *head == p) {
            if state.field_occupation.get(p.as_dim()) != Some(&Some(state.my_id))
            {
                result.snake_head_distances.push(dist);
            }
        }
        if !state.is_occupied(p.clone()) {
            result.size += 1;
            result.sum_x += p.x;
            result.sum_y += p.y;
            result.wide_room_score += 0.75f32.powf(dist as f32);
            for direction in iter_directions() {
                let next_pos = move_by_direction(&p, &direction, &state.game_size);
                if !visited.contains(&next_pos) {
                    visited.insert(next_pos.clone());
                    queue.push_back((dist + 1, next_pos));
                }
            }
        } else {
            result.bounding_snakes.insert(state.field_occupation.get(p.as_dim()).unwrap().unwrap().clone());
        }
    }
    return result;
}

fn evaluate_empty_space(state: &EmptySpaceState) -> f32 {
    -(state.size as f32) * (state.bounding_snakes.len() as f32).powf(0.25) / (state.snake_head_distances.len() as f32 + 1.0).sqrt()
}

fn evaluate_direction(pos: &Position, space: &EmptySpaceState, state: &State) -> f32 {
    info!("Wide space score: {}", space.wide_room_score / 20.0);
    space.snake_head_distances.iter()
        .map(|dist| 1.0 / *dist as f32)
        .sum::<f32>()
    - if has_wall(pos, state) { 0.3 } else { 0.0 }
    - space.wide_room_score / 20.0
}
