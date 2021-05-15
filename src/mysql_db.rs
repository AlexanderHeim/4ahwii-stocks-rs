use std::{collections::BTreeMap, str::FromStr};

use mysql::{Pool, PooledConn, chrono::NaiveDate, params, prelude::Queryable};
use mysql_common::bigdecimal::BigDecimal;

use crate::{alphavantage::AlphaVantage, config::Config, timeseries::TimeSeries};

pub struct Database {
    pub pool: Pool,
    pub conn: PooledConn,
}

impl Database {
    pub fn from_config(config: &Config) -> Self {
        let pool = match Pool::new(&config.mysql_url) {
            Ok(pool) => pool,
            Err(error) => panic!("Unable to create mysql pool: {}", error)
        };
        let conn = match pool.get_conn() {
            Ok(conn) => conn,
            Err(error) => panic!("Unable to create connection from pool: {}", error)
        };

        Database {
            pool,
            conn,
        }
    }

    pub fn update(&mut self, symbol: &str, alphavantage: &AlphaVantage) {
        // Create raw and adjusted table if not exists
        self.conn.query_drop(format!("
            create table if not exists {}_raw (
            entry_date DATE not null primary key,
            close_value decimal(11, 2) not null,
            split_coefficient decimal(4, 2) not null)", 
            symbol)).unwrap();
        
        self.conn.query_drop(format!("
            create table if not exists {}_adjusted (
            entry_date DATE not null primary key,
            close_value decimal(11, 2) not null,
            split_coefficient decimal(4, 2) not null)", 
            symbol)).unwrap();

        self.conn.query_drop(format!("
            create table if not exists {}_200avg (
            entry_date DATE not null primary key,
            close_value decimal(11, 2) not null)", 
            symbol)).unwrap();
            
        //Check if raw table is empty, if it is, do a large fetch
        let size: i32 = self.conn.query_first(format!("SELECT COUNT(*) FROM {}_raw", symbol)).expect("Couldn't query count(*) from table!").unwrap();
        if size == 0 {
            // Insert raw data into raw table
            let mut full = alphavantage.fetch_daily(symbol, false);
            let stmt = self.conn.prep(format!("INSERT INTO {}_raw (entry_date, close_value, split_coefficient) VALUES (:entry_date, :close_value, :split_coefficient)", symbol)).unwrap();
            self.conn.exec_batch(stmt, full.entries.iter().map( |p| params! {
                "entry_date" => p.0,
                "close_value" => p.1.0.to_string(),
                "split_coefficient" => p.1.1,
            })).expect("Couldn't insert entries into database!");

            // Reset then Insert adjusted data into adjusted table
            self.conn.query_drop(format!("DELETE FROM {}_adjusted", symbol)).unwrap();
            full.correct_splits();
            let stmt = self.conn.prep(format!("INSERT INTO {}_adjusted (entry_date, close_value, split_coefficient) VALUES (:entry_date, :close_value, :split_coefficient)", symbol)).unwrap();
            self.conn.exec_batch(stmt, full.entries.iter().map( |p| params! {
                "entry_date" => p.0,
                "close_value" => p.1.0.to_string(),
                "split_coefficient" => p.1.1,
            })).expect("Couldn't insert entries into database!");

            //Calculate all 200avgs
            for i in 200..full.entries.len() {
                self.conn.query_drop(format!("Insert into {}_200avg (entry_date, close_value) values('{}', (with temp as ( select close_value from {}_adjusted where entry_date <= '{}' order by entry_date desc limit 200) select avg(close_value) from temp))", symbol, full.nth(i).0, symbol, full.nth(i).0)).expect("Couldnt calculate 200avg!");
            }
            
        } else {
            // Else insert only new data
            let mut timeseries = alphavantage.fetch_daily(symbol, true);
            // Check if compact covers the new data
            let max_date: NaiveDate = self.conn.query(format!("SELECT MAX(entry_date) FROM {}_raw", symbol)).unwrap()[0];
            if max_date < timeseries.nth(0).0 {
                timeseries = alphavantage.fetch_daily(symbol, false);
            }
            // Insert new data
            let stmt = self.conn.prep(format!("INSERT INTO {}_raw (entry_date, close_value, split_coefficient) VALUES (:entry_date, :close_value, :split_coefficient)", symbol)).unwrap();
            let mut new_count_entries = 0;
            for i in (0..timeseries.entries.len()).rev() {
                if max_date < timeseries.nth(i).0 {
                    self.conn.exec_drop(&stmt, params! {
                        "entry_date" => timeseries.nth(i).0,
                        "close_value" => timeseries.nth(i).1.0.to_string(),
                        "split_coefficient" => timeseries.nth(i).1.1,
                    }).expect("Couldn't insert new data!");
                } else {
                    new_count_entries = timeseries.entries.len() - i - 1;
                    break
                }
            }
            // Insert into adjusted and adjust whole table in case of a new split
            let actually_new_entries = &timeseries.between(timeseries.entries.len()-new_count_entries, timeseries.entries.len());
            let stmt = self.conn.prep(format!("REPLACE INTO {}_adjusted (entry_date, close_value, split_coefficient) VALUES (:entry_date, :close_value, :split_coefficient)", symbol)).unwrap();
            let mut new_splits: BTreeMap<NaiveDate, f32> = BTreeMap::new();
            // Insert actually new entries into adjusted and 200avg
            actually_new_entries.iter().for_each(|x| {
                if x.1.1.unwrap() != 1.0 {
                    new_splits.insert(x.0, x.1.1.unwrap());
                }
                self.conn.exec_drop(&stmt, params! {
                    "entry_date" => x.0,
                    "close_value" => x.1.0.to_string(),
                    "split_coefficient" => x.1.1,
                }).expect("Couldn't insert new data!");
                self.conn.query_drop(format!("Insert into {}_200avg (entry_date, close_value) values('{}', (with temp as ( select close_value from tsla_adjusted where entry_date <= '{}' order by entry_date desc limit 200) select avg(close_value) from temp))", symbol, x.0, x.0)).expect("Couldnt calculate 200avg!");
            });
            //If a split occurs, updat adjusted for all of them
            for d in new_splits.keys() {
                let date: &str = &d.to_string();
                self.conn.query_drop(format!("UPDATE {}_adjusted SET close_value = close_value/{} where entry_date <= '{}'", symbol, new_splits.get(d).unwrap().to_string(), date)).expect("Couldn't execute update query!");
            }
            //If splits occured, update the 200 avgs
            if !new_splits.is_empty() {
                let result: Vec<NaiveDate> = self.conn.query(format!("SELECT entry_date from {}_adjusted", symbol)).unwrap();
                for d in result {
                    self.conn.query_drop(format!("update {}_200avg set close_value = (with temp as ( select close_value from tsla_adjusted where entry_date <= '{}' order by entry_date desc limit 200) select avg(close_value) from temp) where entry_date <= '{}'", symbol, d.to_string(), d.to_string())).expect("Couldn't update 200avgs!")
                }
            }
        }
    }

    pub fn get_timeseries_between(&mut self, symbol: &str, table_name: &str, start_date: NaiveDate, end_date: NaiveDate) -> TimeSeries {

        let mut entries: BTreeMap<NaiveDate, (BigDecimal, Option<f32>)> = BTreeMap::new();

        let result: Vec<(NaiveDate, String, Option<f32>)> = match self.conn.exec(format!("SELECT * FROM {} WHERE entry_date >= :start_date and entry_date <= :end_date", table_name), params! { "start_date" => start_date, "end_date" => end_date }) {
            Ok(result) => result,
            Err(e) => panic!("Couldn't query the timeseries form database: {}", e),
        };

        for i in result {
            entries.insert(i.0, (BigDecimal::from_str(&i.1).unwrap(), i.2));
        }

        TimeSeries {
            equity_name: String::from_str(symbol).unwrap(),
            entries,
        }
    }
}