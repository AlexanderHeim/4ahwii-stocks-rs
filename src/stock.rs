use crate::{alphavantage::AlphaVantage, buy::Backtest, config::Config, mysql_db::{Database}, stockplotter::StockPlotter};

pub struct StockRS {
    pub alphavantage: AlphaVantage,
    pub database: Database,
    pub stockplotter: StockPlotter,
    pub stocks: Vec<String>,
    pub backtest: Backtest,
}

impl StockRS {
    pub fn from_config(config: &Config) -> Self {
        let alphavantage = AlphaVantage::with_key(&config.key);
        let database = Database::from_config(config);
        let stockplotter = StockPlotter::from_config(config);
        let backtest = Backtest::from_config(config);

        StockRS {
            alphavantage,
            database,
            stockplotter,
            stocks: config.stocks.clone(),
            backtest,
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

    pub fn backtest(&mut self) {
        let stocks = &self.stocks;
        for s in stocks {
            self.backtest.full_test(&mut self.database, s, self.stockplotter.start_date, self.stockplotter.end_date);
        }
    }
}