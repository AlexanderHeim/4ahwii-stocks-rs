use std::collections::BTreeMap;

use mysql::chrono::{NaiveDate};
use mysql_common::bigdecimal::BigDecimal;

pub struct TimeSeries {
    pub equity_name: String,
    pub entries: BTreeMap<NaiveDate, (BigDecimal, Option<f32>)>,
}


impl TimeSeries {
    pub fn correct_splits(&mut self) {
        for i in &self.entries.clone() {
            if i.1.1.is_some() && i.1.1.unwrap() != 1.0 {
                self.entries.range_mut(..i.0).for_each(|x| {
                    x.1.0 = &x.1.0 / i.1.1.unwrap();
                });
            }
        }
    }

    pub fn nth(&self, index: usize) -> (NaiveDate, (BigDecimal, Option<f32>)) {
        let date = self.entries.keys().nth(index).unwrap();
        let entry = self.entries.get(date).unwrap();
        (*date, (entry.0.clone(), entry.1))
    }

    pub fn between(&self, start: usize, end: usize) -> Vec<(NaiveDate, (BigDecimal, Option<f32>))> {
        if start >= self.entries.len() || end >= self.entries.len() || start == end {
            return Vec::new();
        }
        let date1 = self.entries.keys().nth(start).unwrap();
        let date2 = self.entries.keys().nth(end).unwrap();
        let mut result: Vec<(NaiveDate, (BigDecimal, Option<f32>))> = Vec::new();
        for i in self.entries.iter().filter(|x| { x.0 >= date1 && x.0 < date2}) {
            let d = *i.0;
            let c = i.1.0.clone();
            let s = i.1.1;
            result.push((d, (c, s)));
        }
        result
    }

    pub fn get_max_close(&self) -> BigDecimal {
        self.entries.values().max_by(|x, y| x.0.cmp(&y.0)).unwrap().0.clone()
    }
}