use config::Config;
use crate::stock::StockRS;


mod config;
mod stockplotter;
mod timeseries;
mod mysql_db;
mod alphavantage;
mod stock;

fn main() {
    let config = Config::read_config();
    println!("{:#?}", config);
    let mut stocks = StockRS::from_config(&config);
    stocks.update_db();
    stocks.plot();
}
