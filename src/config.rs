use std::{fs::File, io::Read};

use toml::Value;

#[derive(Debug)]
pub struct Config {
    key: String,
}

impl Config {
    pub fn parse_config(file: &mut File) -> Self {
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {}
            Err(error) => panic!("Error reading config file: {}", error)
        }

        let config_toml = match contents.parse::<Value>() {
            Ok(toml) => toml,
            Err(error) => panic!("Please check your config.toml syntax: {}", error)
        };

        let key = match config_toml.get("key") {
            Some(key) => key.as_str().unwrap(),
            None => &"asd"
        };
    
        Config {
            key: String::from(key),
        }
    }

    
}