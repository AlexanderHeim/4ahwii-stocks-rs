use mysql::{Pool, PooledConn, params, prelude::Queryable, time::Date};
use mysql_common::{bigdecimal::BigDecimal};
use plotters::{prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea, LineSeries, PathElement}, style::{BLUE, IntoFont, WHITE}};
use std::{collections::BTreeMap, ops::Mul, str::FromStr, time::Duration};
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
        series.sort_by(|a, b| a.date.cmp(&b.date));
        TimeSeriesRaw {
            name: String::from(symbol),
            data: series,
        }
    }

    pub fn update(&mut self, symbol: &str) {
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
            let mut full = self.fetch_daily(symbol, false);
            let stmt = self.conn.prep(format!("INSERT INTO {}_raw (entry_date, close_value, split_coefficient) VALUES (:entry_date, :close_value, :split_coefficient)", symbol)).unwrap();
            self.conn.exec_batch(stmt, full.data.iter().map( |p| params! {
                "entry_date" => p.date,
                "close_value" => p.close.to_string(),
                "split_coefficient" => p.split_coefficient,
            })).expect("Couldn't insert entries into database!");

            // Reset then Insert adjusted data into adjusted table
            self.conn.query_drop(format!("DELETE FROM {}_adjusted", symbol)).unwrap();
            Client::adjust_timeseries(&mut full);
            let stmt = self.conn.prep(format!("INSERT INTO {}_adjusted (entry_date, close_value, split_coefficient) VALUES (:entry_date, :close_value, :split_coefficient)", symbol)).unwrap();
            self.conn.exec_batch(stmt, full.data.iter().map( |p| params! {
                "entry_date" => p.date,
                "close_value" => p.close.to_string(),
                "split_coefficient" => p.split_coefficient,
            })).expect("Couldn't insert entries into database!");

            //Calculate all 200avgs
            for i in 200..full.data.len() {
                self.conn.query_drop(format!("Insert into {}_200avg (entry_date, close_value) values('{}', (with temp as ( select close_value from {}_adjusted where entry_date <= '{}' order by entry_date desc limit 200) select avg(close_value) from temp))", symbol, full.data[i].date, symbol, full.data[i].date)).expect("Couldnt calculate 200avg!");
            }
            
        } else {
            // Else insert only new data
            let mut timeseries = self.fetch_daily(symbol, true);
            // Check if compact covers the new data
            let max_date: Date = self.conn.query(format!("SELECT MAX(entry_date) FROM {}_raw", symbol)).unwrap()[0];
            if max_date < timeseries.data[0].date {
                timeseries = self.fetch_daily(symbol, false);
            }
            // Insert new data
            let stmt = self.conn.prep(format!("INSERT INTO {}_raw (entry_date, close_value, split_coefficient) VALUES (:entry_date, :close_value, :split_coefficient)", symbol)).unwrap();
            let mut new_count_entries = 0;
            for i in (0..timeseries.data.len()).rev() {
                if max_date < timeseries.data[i].date {
                    self.conn.exec_drop(&stmt, params! {
                        "entry_date" => timeseries.data[i].date,
                        "close_value" => timeseries.data[i].close.to_string(),
                        "split_coefficient" => timeseries.data[i].split_coefficient,
                    }).expect("Couldn't insert new data!");
                } else {
                    new_count_entries = timeseries.data.len() - i - 1;
                    break
                }
            }
            // Insert into adjusted and adjust whole table in case of a new split
            let actually_new_entries = &timeseries.data[timeseries.data.len()-new_count_entries..timeseries.data.len()];
            let stmt = self.conn.prep(format!("REPLACE INTO {}_adjusted (entry_date, close_value, split_coefficient) VALUES (:entry_date, :close_value, :split_coefficient)", symbol)).unwrap();
            let mut new_splits: BTreeMap<Date, f32> = BTreeMap::new();
            // Insert actually new entries into adjusted and 200avg
            actually_new_entries.iter().for_each(|x| {
                if x.split_coefficient != 1.0 {
                    new_splits.insert(x.date, x.split_coefficient);
                }
                self.conn.exec_drop(&stmt, params! {
                    "entry_date" => x.date,
                    "close_value" => x.close.to_string(),
                    "split_coefficient" => x.split_coefficient,
                }).expect("Couldn't insert new data!");
                self.conn.query_drop(format!("Insert into {}_200avg (entry_date, close_value) values('{}', (with temp as ( select close_value from tsla_adjusted where entry_date <= '{}' order by entry_date desc limit 200) select avg(close_value) from temp))", symbol, x.date, x.date)).expect("Couldnt calculate 200avg!");
            });
            //If a split occurs, updat adjusted for all of them
            for d in new_splits.keys() {
                let date: &str = &d.to_string();
                self.conn.query_drop(format!("UPDATE {}_adjusted SET close_value = close_value/{} where entry_date <= '{}'", symbol, new_splits.get(d).unwrap().to_string(), date)).expect("Couldn't execute update query!");
            }
            //If splits occured, update the 200 avgs
            if !new_splits.is_empty() {
                let result: Vec<Date> = self.conn.query(format!("SELECT entry_date from {}_adjusted", symbol)).unwrap();
                for d in result {
                    self.conn.query_drop(format!("update {}_200avg set close_value = (with temp as ( select close_value from tsla_adjusted where entry_date <= '{}' order by entry_date desc limit 200) select avg(close_value) from temp) where entry_date <= '{}'", symbol, d.to_string(), d.to_string())).expect("Couldn't update 200avgs!")
                }
            }
        }
    }

    //TODO FINISH THIS
    pub fn print_plot(&mut self, symbol: &str) {
        match std::fs::create_dir(format!("{}", symbol)) {
            Ok(_) => {}
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => panic!("Couldn't create directory: {}", e),
            }
        }
        let path = &format!("./{}/{}.png", symbol, symbol);
        let root = BitMapBackend::new(&path, (1000, 700)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .caption("IBM Timeseries", ("sans-serif", 50.0).into_font())
            .build_cartesian_2d(1..40, 2..40).expect("Couldn't create chart!");

        chart
            .draw_series(LineSeries::new(
                (0..=100).map(|x| (x as f32 / 10.0, (1.02f32).powf(x as f32 * x as f32 / 10.0))),
                &BLUE,
            )).unwrap()
            .label("y = 1.02^x^2")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    
        chart.configure_mesh().light_line_style(&WHITE).draw().unwrap();
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

    pub fn get_splits_in_timeseries(timeseries: &TimeSeriesRaw) -> BTreeMap<Date, f32> {
        let mut map: BTreeMap<Date, f32> = BTreeMap::new();
        timeseries.data.iter().for_each(|x| {
            if x.split_coefficient != 1.0 {
                map.insert(x.date, x.split_coefficient);
            }
        });
        map
    }

    pub fn adjust_saved_timeseries(&mut self, tablename: &str, splits: BTreeMap<Date, f32>) {
        for d in splits.keys() {
            let date: &str = &d.to_string();
            self.conn.query_drop(format!("UPDATE {} SET close_value = close_value/{} where entry_date <= '{}'", tablename, splits.get(d).unwrap().to_string(), date)).expect("Couldn't execute update query!");
        }
    }

    pub fn get_saved_timeseries(&mut self, tablename: &str, symbol: &str) -> TimeSeriesRaw {
        let result: Vec<(Date, String, f32)> = self.conn.query(format!("SELECT * FROM {}", tablename)).expect(&format!("Couldn't query from {}", tablename));
        let mut data: Vec<DayDataRaw> = Vec::new();
        for i in 0..result.len() {
            data.push(DayDataRaw {
                date: result[i].0,
                close: BigDecimal::from_str(&result[i].1).unwrap(),
                split_coefficient: result[i].2
            });
        }
        TimeSeriesRaw {
            name: String::from(symbol),
            data,
        }
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
