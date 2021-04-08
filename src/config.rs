use std::io::{ErrorKind, Read};

use toml::Value;

#[derive(Debug)]
pub struct Config {
    pub key: String,
    pub mysql_url: String,
}

impl Config {
    pub fn read_config() -> Self {
        let mut config_file = match std::fs::File::open("config.toml") {
            Ok(config) => config,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => match std::fs::File::create("config.toml") {
                    Ok(fc) => fc,
                    Err(e) => panic!("Problem creating the config.toml file: {:?}", e),
                },
                other_error => panic!("Problem opening the config file: {:?}", other_error)
            }
        };

        let mut contents = String::new();
        match config_file.read_to_string(&mut contents) {
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

        let mysql_url = match config_toml.get("mysql_url") {
            Some(url) => url.as_str().unwrap(),
            None => panic!("You need to specify a url in your config.toml! Example: mysql_url = 'mysql://root:password@localhost:3307/db_name'")
        };
    
        Config {
            key: String::from(key),
            mysql_url: String::from(mysql_url),
        }
    }
    
}