use gpn21_tron::Config;
use log::error;
use std::io::BufReader;
use std::{env, fs};

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let config_file = if args.len() > 1 {
        &args[1]
    } else {
        "config.toml"
    };

    let config_string = fs::read_to_string(config_file).unwrap();
    let config: Config = toml::from_str(&config_string).unwrap();
    let mut rng = rand::thread_rng();
    loop {
        let mut stream = gpn21_tron::get_connection(&config.server);
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let res = gpn21_tron::game_loop(&config, &mut stream, &mut reader, &mut rng);
        if let Err(e) = res {
            error!("IO error: {}", e);
        }
    }
}
