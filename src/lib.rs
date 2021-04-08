use mysql::time::Date;
use mysql_common::bigdecimal::BigDecimal;
use std::{str::FromStr, time::Instant};

pub struct Client {
    pub key: String,
}

impl Client {
    pub fn fetch_daily(&self, symbol: &str, compact: bool) -> Vec<DayDataRaw> {
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
        series
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
