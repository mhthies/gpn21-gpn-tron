use crate::algorithm::State;
use crate::{MoveDirection, Position};
use ordered_float::OrderedFloat;

pub fn move_by_direction(pos: &Position, dir: &MoveDirection, game_size: &Position) -> Position {
    match dir {
        MoveDirection::Up => Position {
            x: pos.x,
            y: (game_size.y + pos.y - 1) % game_size.y,
        },
        MoveDirection::Right => Position {
            x: (pos.x + 1) % game_size.x,
            y: pos.y,
        },
        MoveDirection::Down => Position {
            x: pos.x,
            y: (pos.y + 1) % game_size.y,
        },
        MoveDirection::Left => Position {
            x: (game_size.x + pos.x - 1) % game_size.x,
            y: pos.y,
        },
    }
}

pub fn point_to_float_point_distance(p: &Position, x2: f32, y2: f32, game_size: &Position) -> f32 {
    return [
        OrderedFloat(((p.x as f32 - x2).powi(2) + (p.y as f32 - y2).powi(2)).sqrt()),
        OrderedFloat(
            ((p.x as f32 - x2).powi(2) + ((p.y + game_size.y) as f32 - y2).powi(2)).sqrt(),
        ),
        OrderedFloat(
            (((p.x + game_size.x) as f32 - x2).powi(2) + (p.y as f32 - y2).powi(2)).sqrt(),
        ),
        OrderedFloat(
            (((p.x + game_size.x) as f32 - x2).powi(2) + ((p.y + game_size.y) as f32 - y2).powi(2))
                .sqrt(),
        ),
    ]
    .iter()
    .min()
    .unwrap()
    .0;
}

pub fn point_to_point_distance(p1: &Position, p2: &Position, game_size: &Position) -> f32 {
    point_to_float_point_distance(p1, p2.x as f32, p2.y as f32, game_size)
}

pub fn has_wall(pos: &Position, game_state: &State) -> bool {
    [
        MoveDirection::Up,
        MoveDirection::Down,
        MoveDirection::Left,
        MoveDirection::Right,
    ]
    .iter()
    .map(|d| move_by_direction(pos, d, &game_state.game_size))
    .filter(|p| !game_state.player_heads.values().any(|head| *p == *head))
    .map(|p| game_state.is_occupied(p))
    .any(|b| b)
}

pub fn has_neighbour_head(pos: &Position, game_state: &State) -> bool {
    [
        MoveDirection::Up,
        MoveDirection::Down,
        MoveDirection::Left,
        MoveDirection::Right,
    ]
    .iter()
    .map(|d| move_by_direction(pos, d, &game_state.game_size))
    .any(|p| {
        game_state
            .player_heads
            .iter()
            .filter(|(player, _head)| **player != game_state.my_id)
            .any(|(_player, head)| p == *head)
    })
}

pub fn distance_to_next_opponent_head(pos: &Position, game_state: &State) -> Option<u32> {
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    visited.insert(pos.clone());
    queue.push_back((0usize, pos.clone()));

    while let Some((dist, p)) = queue.pop_front() {
        if game_state
            .player_heads
            .iter()
            .filter(|(player, _head)| **player != game_state.my_id)
            .any(|(_player, head)| *head == p)
        {
            return Some(dist as u32);
        }
        if !game_state.is_occupied(p.clone()) {
            for direction in [
                MoveDirection::Up,
                MoveDirection::Down,
                MoveDirection::Left,
                MoveDirection::Right,
            ] {
                let next_pos = move_by_direction(&p, &direction, &game_state.game_size);
                if !visited.contains(&next_pos) {
                    visited.insert(next_pos.clone());
                    queue.push_back((dist + 1, next_pos));
                }
            }
        }
    }
    return None;
}
