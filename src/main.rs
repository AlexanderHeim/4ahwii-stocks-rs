use client::Client;
use config::Config;
use mysql::time::Date;


mod config;
mod client;

fn main() {
    let config = Config::read_config();
    println!("{:#?}", config);
    let mut client = Client::from_config(&config);

    let mut ts = client.fetch_daily("tsla", false);
    client.update_timeseries_raw(&mut ts);

    /*let client = Client {
        key: String::from("asd"),
    };
    let ts = client.fetch_daily("TSLA", true);
    println!("{:#?}", ts);*/

}
