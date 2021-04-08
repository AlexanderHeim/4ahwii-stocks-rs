use mysql::{Pool, PooledConn, params, prelude::Queryable, time::Date};
use mysql_common::{bigdecimal::BigDecimal};
use std::{str::FromStr};

use crate::config::Config;



pub struct Client {
    pub key: String,
    pub pool: Pool,
    pub conn: PooledConn,
}

impl Client {
    pub fn from_config(config: &Config) -> Self {
        let pool = match Pool::new(&config.mysql_url) {
            Ok(pool) => pool,
            Err(error) => panic!("Unable to create mysql pool: {}", error)
        };
        let conn = match pool.get_conn() {
            Ok(conn) => conn,
            Err(error) => panic!("Unable to create connection from pool: {}", error)
        };

        Client {
            key: String::from(&config.key),
            pool,
            conn,
        }
    }

    pub fn fetch_daily(&self, symbol: &str, compact: bool) -> TimeSeriesRaw {
        let url;
        if compact {
            url = format!("https://www.alphavantage.co/query?function=TIME_SERIES_DAILY_ADJUSTED&outputsize=compact&symbol={}&apikey={}", symbol, self.key);
        } else {
            url = format!("https://www.alphavantage.co/query?function=TIME_SERIES_DAILY_ADJUSTED&outputsize=full&symbol={}&apikey={}", symbol, self.key);
        }
        
        let json = reqwest::blocking::get(&url).expect(&format!("Couldn't get json from Alphavantage! Request url: {}", &url)).text().unwrap();
        let parsed = json::parse(&json).unwrap();
        let mut series: Vec<DayDataRaw> = Vec::new();
        parsed["Time Series (Daily)"].entries().for_each(|x| {
            let entry_json =  &parsed["Time Series (Daily)"][x.0];
            let entry = DayDataRaw {
                date: Date::parse(x.0, "%F").unwrap(),
                close: BigDecimal::from_str(&entry_json["4. close"].to_string()).unwrap(),
                split_coefficient: entry_json["8. split coefficient"].to_string().parse().unwrap(),
            };
            series.push(entry);
        });

        TimeSeriesRaw {
            name: String::from(symbol),
            data: series,
        }
    }

    pub fn update_timeseries_raw(&mut self, timeseries: TimeSeriesRaw) {
        todo!("actually update, not insert");
        println!("{}", timeseries.data[1].close.to_string());
        match self.conn.exec_batch(format!("INSERT INTO {}_raw (entry_date, close_value, split_coefficient) VALUES (:date, :close, :split_coefficient)", timeseries.name), 
            timeseries.data.iter().map(|p| params! {
                "date" => p.date,
                "close" => p.close.to_string(),
                "split_coefficient" => p.split_coefficient,
            })) {
                Ok(_) => {}
                Err(error) => panic!("Unable to execute batch (update timeseries raw): {}", error)
            }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DayDataEx {
    pub date: Date,
    pub close: BigDecimal,
    pub close_avg200: Option<BigDecimal>,
}

#[derive(Debug, PartialEq)]
pub struct DayDataRaw {
    pub date: Date,
    pub close:  BigDecimal,
    pub split_coefficient: f32,
}

#[derive(Debug, PartialEq)]
pub struct TimeSeriesRaw {
    pub name: String,
    pub data: Vec<DayDataRaw>,
}
