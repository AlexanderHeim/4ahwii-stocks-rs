use mysql::chrono::NaiveDate;
use mysql_common::bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use crate::{config::Config, mysql_db::Database};

pub struct Backtest {
    start_depot: BigDecimal,
    avg200_range: f32,
}

impl Backtest {
    pub fn from_config(config: &Config) -> Self {
        Backtest {
            start_depot: config.start_depot.clone(),
            avg200_range: config.avg200_range,
        }
    }

    pub fn full_test(&self, db: &mut Database, symbol: &str, start_date: NaiveDate, end_date: NaiveDate) {
        println!("Backtesting {}", symbol);
        let mut depot = Depot::new(&self.start_depot);
        backtest_normal(db, symbol, &mut depot, start_date, end_date);
        println!("Buy-And-Hold Endvalue: {}€", depot.money);

        let mut depot = Depot::new(&self.start_depot);
        backtest_avg200(db, symbol, &mut depot, start_date, end_date);
        println!("200 Average Endvalue: {}€", depot.money);

        let mut depot = Depot::new(&self.start_depot);
        backtest_avg200_range(db, symbol, &mut depot, start_date, end_date, self.avg200_range);
        println!("200 Average Range Endvalue: {}€", depot.money);
        println!("");
    }
}

fn backtest_normal(db: &mut Database, symbol: &str, depot: &mut Depot, start_date: NaiveDate, end_date: NaiveDate) {
    let ts = db.get_timeseries_between(symbol, &format!("{}_adjusted", symbol), start_date, end_date);

    let first_day = ts.entries.iter().nth(0).unwrap();
    let last_day = ts.entries.iter().last().unwrap();
    depot.full_buy(&first_day.1.0);
    depot.full_sell(&last_day.1.0);
}

fn backtest_avg200(db: &mut Database, symbol: &str, depot: &mut Depot, start_date: NaiveDate, end_date: NaiveDate) {
    let ts = db.get_timeseries_between(symbol, &format!("{}_adjusted", symbol), start_date, end_date);
    let ts_200 = db.get_timeseries_between(symbol, &format!("{}_200avg", symbol), start_date, end_date);

    for day in &ts.entries {
        let day_200 = match ts_200.entries.get(&day.0) {
            Some(s) => s,
            None => continue,
        };
        if day.1.0 > day_200.0 {
            depot.full_buy(&day.1.0);
        } else {
            depot.full_sell(&day.1.0);
        }
        
    }
    depot.full_sell(&ts.entries.into_iter().last().unwrap().1.0);
}

fn backtest_avg200_range(db: &mut Database, symbol: &str, depot: &mut Depot, start_date: NaiveDate, end_date: NaiveDate, range: f32) {
    let ts = db.get_timeseries_between(symbol, &format!("{}_adjusted", symbol), start_date, end_date);
    let ts_200 = db.get_timeseries_between(symbol, &format!("{}_200avg", symbol), start_date, end_date);

    for day in &ts.entries {
        let day_200 = match ts_200.entries.get(&day.0) {
            Some(s) => s,
            None => continue,
        };
        if &day.1.0/&day_200.0 > BigDecimal::from_f32(range).unwrap() {
            depot.full_buy(&day.1.0);
        } else if  &day.1.0/&day_200.0 < BigDecimal::from_f32(range).unwrap() {
            depot.full_sell(&day.1.0);
        }
        
    }
    depot.full_sell(&ts.entries.into_iter().last().unwrap().1.0);
}


pub struct Depot {
    pub money: BigDecimal,
    pub shares: u32,
}

impl Depot {
    pub fn new(money: &BigDecimal) -> Self {
        Depot {
            money: money.clone(),
            shares: 0,
        }
    }

    pub fn buy(&mut self, amount: u32, price: &BigDecimal) {
        self.money -= BigDecimal::from(amount) * price;
        self.shares += amount;
    }

    pub fn sell(&mut self, amount: u32, price: &BigDecimal) {
        self.money += BigDecimal::from(amount) * price;
        self.shares -= amount;
    }

    pub fn full_buy(&mut self, price: &BigDecimal) {
        let amount = (&self.money / price).to_u32().unwrap();
        self.buy(amount, price);
    }

    pub fn full_sell(&mut self, price: &BigDecimal) {
        let amount = self.shares;
        self.sell(amount, price);
    }
}


#[cfg(test)]
mod tests {
    use mysql_common::bigdecimal::BigDecimal;

    use super::Depot;

    #[test]
    fn works() {
        let mut depot = Depot::new(&BigDecimal::from(100000));
        depot.full_buy(&BigDecimal::from(100));
        assert_eq!(depot.money, BigDecimal::from(0));
        assert_eq!(depot.shares, 1000);

        let mut depot = Depot::new(&BigDecimal::from(100000));
        depot.full_buy(&BigDecimal::from(342));
        assert_eq!(depot.money, BigDecimal::from(136));
        assert_eq!(depot.shares, 292);
    }
}