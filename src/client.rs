use crate::{MoveDirection, Position};
use log::{debug, warn};
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

#[derive(Default, Clone, Eq, PartialEq, Hash, Copy, Debug)]
pub struct PlayerId(u32);

#[derive(Debug)]
pub enum Command<'a> {
    Join(&'a str, &'a str),
    Move(MoveDirection),
    Chat(&'a str),
}

pub enum Answer {
    Motd(String),
    Error(String),
    Pos(PlayerId, Position),
    Win(u32, u32),
    Lose(u32, u32),
    Game(Position, PlayerId),
    Tick,
    Die(Vec<PlayerId>),
    Message(PlayerId, String),
    Player(PlayerId, String),
}

pub fn get_answer(reader: &mut BufReader<TcpStream>) -> io::Result<Option<Answer>> {
    let mut command = String::new();
    reader.read_line(&mut command)?;
    debug!("Received answer: {}", command.trim());
    let mut parts = command.trim().split("|");
    Ok(match parts.next().unwrap() {
        "motd" => Some(Answer::Motd(parts.next().unwrap_or("").to_owned())),
        "error" => Some(Answer::Error(parts.next().unwrap_or("").to_owned())),
        "pos" => Some(Answer::Pos(
            PlayerId(parts.next().unwrap_or("0").parse().unwrap_or(0)),
            Position {
                x: parts.next().unwrap_or("0").parse().unwrap_or(0),
                y: parts.next().unwrap_or("0").parse().unwrap_or(0),
            },
        )),
        "win" => Some(Answer::Win(
            parts.next().unwrap_or("0").parse().unwrap_or(0),
            parts.next().unwrap_or("0").parse().unwrap_or(0),
        )),
        "lose" => Some(Answer::Lose(
            parts.next().unwrap_or("0").parse().unwrap_or(0),
            parts.next().unwrap_or("0").parse().unwrap_or(0),
        )),
        "game" => Some(Answer::Game(
            Position {
                x: parts.next().unwrap_or("0").parse().unwrap_or(0),
                y: parts.next().unwrap_or("0").parse().unwrap_or(0),
            },
            PlayerId(parts.next().unwrap_or("0").parse().unwrap_or(0)),
        )),
        "tick" => Some(Answer::Tick),
        "die" => Some(Answer::Die(
            parts.map(|id| PlayerId(id.parse().unwrap_or(0))).collect(),
        )),
        "message" => Some(Answer::Message(
            PlayerId(parts.next().unwrap_or("0").parse().unwrap_or(0)),
            parts.next().unwrap_or("").to_string(),
        )),
        "player" => Some(Answer::Player(
            PlayerId(parts.next().unwrap_or("0").parse().unwrap_or(0)),
            parts.next().unwrap_or("").to_string(),
        )),
        "" => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Empty answer from server.",
            ));
        }
        x => {
            warn!("Unkown message from server: {}", x);
            None
        }
    })
}

pub fn send_command(stream: &mut TcpStream, command: &Command) -> io::Result<()> {
    let data = match command {
        Command::Join(user, password) => format!("join|{}|{}\n", user, password),
        Command::Move(direction) => format!(
            "move|{}\n",
            match direction {
                MoveDirection::Up => "up",
                MoveDirection::Right => "right",
                MoveDirection::Down => "down",
                MoveDirection::Left => "left",
            }
        ),
        Command::Chat(msg) => format!("chat|{}\n", msg),
    };
    debug!("Sending command: {}", data.trim());
    stream.write(data.as_bytes())?;
    stream.flush()?;
    Ok(())
}
