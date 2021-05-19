use std::{collections::BTreeMap, str::FromStr};

use mysql::chrono::{NaiveDate};
use mysql_common::bigdecimal::BigDecimal;

use crate::timeseries::TimeSeries;

pub struct AlphaVantage {
    key: String,
}

impl AlphaVantage {
    pub fn with_key(key: &str) -> Self {
        AlphaVantage {
            key: String::from_str(key).unwrap(),
        }
    }

    pub fn fetch_daily(&self, symbol: &str, compact: bool) -> TimeSeries {
        let url;
        if compact {
            url = format!("https://www.alphavantage.co/query?function=TIME_SERIES_DAILY_ADJUSTED&outputsize=compact&symbol={}&apikey={}", symbol, self.key);
        } else {
            url = format!("https://www.alphavantage.co/query?function=TIME_SERIES_DAILY_ADJUSTED&outputsize=full&symbol={}&apikey={}", symbol, self.key);
        }
        
        let json = reqwest::blocking::get(&url).expect(&format!("Couldn't get json from Alphavantage! Request url: {}", &url)).text().unwrap();
        let parsed = json::parse(&json).unwrap();

        let mut data: BTreeMap<NaiveDate, (BigDecimal, Option<f32>)> = BTreeMap::new();
        
        parsed["Time Series (Daily)"].entries().for_each(|x| {
            let entry_json =  &parsed["Time Series (Daily)"][x.0];
            data.insert(NaiveDate::from_str(x.0).unwrap(),
            (BigDecimal::from_str(&entry_json["4. close"].to_string()).unwrap(),
                   Some(entry_json["8. split coefficient"].to_string().parse().unwrap())));
        });
        if data.is_empty() {
            panic!("KEY INVALID!");
        }
        TimeSeries {
            equity_name: String::from_str(symbol).unwrap(),
            entries: data,
        }
    }
}