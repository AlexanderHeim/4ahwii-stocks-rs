use mysql::{Pool, PooledConn, params, prelude::Queryable, time::Date};
use mysql_common::{bigdecimal::BigDecimal};
use std::{collections::BTreeMap, ops::Mul, str::FromStr};
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

    pub fn update_timeseries_raw(&mut self, timeseries: &mut TimeSeriesRaw) {
        timeseries.data.sort_by(|a, b| a.date.cmp(&b.date));
        let mut end = 0;
        for i in 0..timeseries.data.len() {
            let d = timeseries.data[i].date;
            let result: Vec<(Date, String, f32)> = match self.conn.exec(format!("SELECT * FROM {}_raw WHERE entry_date = :date", timeseries.name), params! {
                "date" => d
            }) {
                Ok(r) => r,
                Err(e) => panic!("Couldn't query entry from database: {}", e),
            };
            if !result.is_empty() {
                end = i;
                break;
            }
        }

        match self.conn.exec_batch(format!("INSERT INTO {}_raw (entry_date, close_value, split_coefficient) VALUES (:date, :close, :split_coefficient)", timeseries.name), 
            timeseries.data.iter().rev().enumerate().filter(|&(i, _)| i <= end).map(|p| params! {
                "date" => p.1.date,
                "close" => p.1.close.to_string(),
                "split_coefficient" => p.1.split_coefficient,
            })) {
                Ok(_) => {}
                Err(error) => panic!("Unable to execute batch (update timeseries raw): {}", error)
            }
    }

    pub fn get_raw_timeseries_between(&mut self, symbol: &str, start_date: &Date, end_date: &Date) -> TimeSeriesRaw {
        let queried: Vec<(Date, String, f32)> = match self.conn.exec(format!("SELECT * FROM {}_raw WHERE entry_date >= :start_date and entry_date <= :end_date", symbol), params! { "start_date" => start_date, "end_date" => end_date }) {
            Ok(result) => result,
            Err(e) => panic!("Couldn't query the raw timeseries form database: {}", e),
        };
        let mut series: Vec<DayDataRaw> = Vec::new();
        for i in 0..queried.len() {
            series.push(DayDataRaw {
                date: queried[i].0,
                close: BigDecimal::from_str(&queried[i].1).unwrap(),
                split_coefficient: queried[i].2,
            })
        }
        println!("{:#?}", series);
        TimeSeriesRaw {
            name: String::from(symbol),
            data: series,
        }
    }

    pub fn adjust_timeseries(timeseries: &mut TimeSeriesRaw) {
        timeseries.data.sort_by(|a, b| a.date.cmp(&b.date));
        for i in (0..timeseries.data.len()).rev() {
            if timeseries.data[i].split_coefficient != 1.0 {
                for j in (0..i).rev() {
                    timeseries.data[j].close = &timeseries.data[j].close / timeseries.data[i].split_coefficient;
                }
            }
        }
    }

    fn get_splits_in_timeseries(timeseries: &TimeSeriesRaw) -> BTreeMap<Date, f32> {
        let mut map: BTreeMap<Date, f32> = BTreeMap::new();
        timeseries.data.iter().for_each(|x| {
            if x.split_coefficient != 1.0 {
                map.insert(x.date, x.split_coefficient);
            }
        });
        map
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct DayDataEx {
    pub date: Date,
    pub close: BigDecimal,
    pub close_avg200: Option<BigDecimal>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct DayDataRaw {
    pub date: Date,
    pub close:  BigDecimal,
    pub split_coefficient: f32,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct TimeSeriesRaw {
    pub name: String,
    pub data: Vec<DayDataRaw>,
}
