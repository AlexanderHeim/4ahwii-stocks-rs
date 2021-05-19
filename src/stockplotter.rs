use mysql::chrono::NaiveDate;
use plotters::{prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea, LineSeries}, style::{BLACK, GREEN, IntoFont, RED, WHITE}};
use mysql_common::bigdecimal::ToPrimitive;

use crate::{config::Config, mysql_db::Database};

pub struct StockPlotter {
    img_width: i32,
    img_height: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl StockPlotter {
    pub fn new(img_width: i32, img_height: i32, start_date: NaiveDate, end_date: NaiveDate) -> Self {
        StockPlotter {
            img_width,
            img_height,
            start_date,
            end_date
        }
    }

    pub fn from_config(config: &Config) -> Self {
        StockPlotter::new(config.img_width, config.img_height, config.start_date, config.end_date)
    } 

    pub fn plot_timeseries(&self, symbol: &str, database: &mut Database)  {
        let ts = database.get_timeseries_between(symbol, &format!("{}_adjusted", symbol), self.start_date, self.end_date);
        let ts2 = database.get_timeseries_between(symbol, &format!("{}_200avg", symbol), self.start_date, self.end_date);
        match std::fs::create_dir(format!("charts")) {
            Ok(_) => {}
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => panic!("Couldn't create directory: {}", e),
            }
        }
        match std::fs::create_dir(format!("charts/{}", symbol)) {
            Ok(_) => {}
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => panic!("Couldn't create directory: {}", e),
            }
        }
        let path = &format!("./charts/{}/{}_{}_{}.png", symbol, symbol, self.start_date, self.end_date);
        
        let root = BitMapBackend::new(&path, (self.img_width as u32, self.img_height as u32)).into_drawing_area();
        if ts.entries.iter().last() < ts2.entries.iter().last() {
            root.fill(&RED).unwrap();
        } else {
            root.fill(&GREEN).unwrap();
        }

        let mut chart = ChartBuilder::on(&root)
            .caption(symbol, ("sans-serif", 50).into_font())
            .x_label_area_size(70)
            .y_label_area_size(70)
            .margin_right(70)
            .build_cartesian_2d(self.start_date..self.end_date, 0 as f32..ts.get_max_close().to_f32().unwrap()).unwrap();

        chart.configure_mesh().draw().unwrap();
        chart.draw_series(LineSeries::new(
            ts.entries.iter().map(|x| (*x.0, x.1.0.to_f32().unwrap())),
            &BLACK,
        )).unwrap();
        chart.draw_series(LineSeries::new(
            ts2.entries.iter().map(|x| (*x.0, x.1.0.to_f32().unwrap())),
            &WHITE,
        )).unwrap();
    }
}