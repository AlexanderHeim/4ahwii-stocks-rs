use mysql::time::Date;
use mysql_common::bigdecimal::BigDecimal;
use std::str::FromStr;

pub struct Client {
    pub key: String,
}

impl Client {
    pub fn large_fetch_daily(&self, symbol: &str) -> Vec<DayData> {
        let url = format!("https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&outputsize=full&symbol={}&apikey={}", symbol, self.key);
        let json = reqwest::blocking::get(&url).expect(&format!("Couldn't get json from Alphavantage! Request url: {}", &url)).text().unwrap();
        let parsed = json::parse(&json).unwrap();
        let mut series: Vec<DayData> = Vec::new();
        parsed["Time Series (Daily)"].entries().for_each(|x| {
            let entry_json =  &parsed["Time Series (Daily)"][x.0]["4. close"];
            let entry = DayData {
                date: Date::parse(x.0, "%F").unwrap(),
                close: BigDecimal::from_str(&entry_json.to_string()).unwrap(),
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

#[derive(Debug, PartialEq, Eq)]
pub struct DayData {
    pub date: Date,
    pub close:  BigDecimal,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TimeSeries {
    pub data: Vec<DayData>,
}