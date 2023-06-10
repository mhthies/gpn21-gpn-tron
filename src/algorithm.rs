use crate::client::{Answer, Command, PlayerId};
use crate::{AlgorithmConfig, Position};
use core::option::Option;
use core::option::Option::{None, Some};
use rand::rngs::ThreadRng;
use std::collections::HashMap;

mod algorithm1;
mod algorithm2;
mod algorithm3;
mod helper;

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
            }
            Answer::Game(size, my_id) => {
                self.my_id = my_id.clone();
                self.field_occupation = ndarray::Array2::from_elem(size.as_dim(), None);
                self.game_size = size.clone();
                self.player_heads.clear();
            }
            Answer::Die(p) => {
                for field in self.field_occupation.iter_mut() {
                    if *field == Some(p.clone()) {
                        *field = None;
                    }
                }
                self.player_heads.remove(p);
            }
            _ => {}
        }
    }

    fn is_occupied(&self, p: Position) -> bool {
        self.field_occupation[p.as_dim()].is_some()
    }
}

pub fn decide_action(
    state: &mut State,
    rng: &mut ThreadRng,
    config: &AlgorithmConfig,
) -> Option<Command<'static>> {
    if state.game_size.x == 0 || state.game_size.y == 0 {
        return None;
    }

    match config.algorithm {
        0 => algorithm1::decide_action(state, rng, config),
        1 => algorithm2::decide_action(state, rng, config),
        2 => algorithm3::decide_action(state, rng, config),
        _ => panic!("Unknown algorithm variant {}", config.algorithm),
    }
}
