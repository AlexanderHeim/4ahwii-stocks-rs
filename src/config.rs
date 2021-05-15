use std::{io::{ErrorKind, Read}, str::FromStr};

use mysql::chrono::{NaiveDate, Utc};
use toml::Value;

#[derive(Debug)]
pub struct Config {
    pub key: String,
    pub mysql_url: String,
    pub img_width: i32,
    pub img_height: i32,
    pub stocks: Vec<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
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

        let img_width = match config_toml.get("img_width") {
            Some(width) => width.as_integer().unwrap() as i32,
            None => 1280 as i32,
        };

        let img_height = match config_toml.get("img_height") {
            Some(height) => height.as_integer().unwrap() as i32,
            None => 720 as i32,
        };

        let _stocks = match config_toml.get("stocks") {
            Some(stocks) => stocks.as_array().unwrap(),
            None => panic!("You need to specify stocks to keep track of in your config.toml! Example: stocks = ['ibm', 'tsla']"),
        };
    
        let mut stocks: Vec<String> = Vec::new();
        for i in 0.._stocks.len() {
            stocks.push(String::from_str(_stocks[i].as_str().unwrap()).unwrap());
        };

        let start_date = match config_toml.get("img_start_date") {
            Some(start_date) => NaiveDate::parse_from_str(start_date.as_str().unwrap(), "%d-%m-%Y").unwrap(),
            None => NaiveDate::from_ymd(2020, 1, 1),
        };

        let end_date = match config_toml.get("img_end_date") {
            Some(end_date) => NaiveDate::parse_from_str(end_date.as_str().unwrap(), "%d-%m-%Y").unwrap(),
            None => Utc::now().date().naive_utc(),
        };

        Config {
            key: String::from(key),
            mysql_url: String::from(mysql_url),
            img_width,
            img_height,
            stocks,
            start_date,
            end_date,
        }
    }
    
}