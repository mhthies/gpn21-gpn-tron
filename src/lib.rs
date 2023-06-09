use crate::algorithm::{decide_action, State};
use crate::client::{send_command, Answer, Command};
use log::{error, info, warn};
use rand::prelude::ThreadRng;
use serde::Deserialize;
use std::io::BufReader;
use std::net::TcpStream;
use std::{io, thread, time};

mod algorithm;
mod client;

#[derive(Default, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    fn as_dim(&self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }
}

#[derive(Clone, Debug)]
pub enum MoveDirection {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub address: String,
}

#[derive(Deserialize)]
struct UserConfig {
    user: String,
    password: String,
}

#[derive(Deserialize)]
pub struct AlgorithmConfig {
    #[serde(default)]
    algorithm: u32,
}

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    user: UserConfig,
    algorithm: AlgorithmConfig,
}

pub fn get_connection(config: &ServerConfig) -> TcpStream {
    loop {
        match TcpStream::connect(&config.address) {
            Ok(s) => return s,
            Err(e) => error!("Could not connect: {}", e),
        }
        thread::sleep(time::Duration::from_millis(200));
    }
}

pub fn game_loop(
    config: &Config,
    stream: &mut TcpStream,
    stream_reader: &mut BufReader<TcpStream>,
    rng: &mut ThreadRng,
) -> io::Result<()> {
    let mut state = State::default();
    info!("Joining game as {}", config.user.user);
    send_command(
        stream,
        &Command::Join(&config.user.user, &config.user.password),
    )?;
    info!("Starting game loop.");
    loop {
        if let Some(answer) = client::get_answer(stream_reader)? {
            match &answer {
                Answer::Motd(msg) => {
                    warn!("Message of the day: {}", msg);
                }
                Answer::Error(msg) => {
                    warn!("Error from Server: {}", msg);
                    if msg.contains("kicked") {
                        return Err(io::Error::from(io::ErrorKind::Other));
                    }
                }
                Answer::Win(_, _) => {
                    warn!("We won!");
                }
                Answer::Lose(_, _) => {
                    warn!("We lost!");
                }
                Answer::Tick => {
                    info!("Tick.");
                    if let Some(command) = decide_action(&mut state, rng, &config.algorithm) {
                        info!("Command: {:?}", command);
                        client::send_command(stream, &command)?;
                    }
                }
                _ => {}
            }
            state.update_from_answer(&answer);
        }
    }
}
