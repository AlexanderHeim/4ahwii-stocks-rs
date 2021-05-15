use crate::{alphavantage::AlphaVantage, config::Config, mysql_db::{Database}, stockplotter::StockPlotter};

pub struct StockRS {
    pub alphavantage: AlphaVantage,
    pub database: Database,
    pub stockplotter: StockPlotter,
    pub stocks: Vec<String>,
}

impl StockRS {
    pub fn from_config(config: &Config) -> Self {
        let alphavantage = AlphaVantage::with_key(&config.key);
        let database = Database::from_config(config);
        let stockplotter = StockPlotter::from_config(config);

        StockRS {
            alphavantage,
            database,
            stockplotter,
            stocks: config.stocks.clone(),
        }
    }

    pub fn update_db(&mut self) {
        let stocks = &self.stocks;
        for i in stocks {
            self.database.update(&i, &self.alphavantage);
        }
    }

    pub fn plot(&mut self) {
        for i in &self.stocks {
            self.stockplotter.plot_timeseries(i, &mut self.database);
        }
    }
}