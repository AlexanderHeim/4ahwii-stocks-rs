use client::Client;
use config::Config;


mod config;
mod client;

fn main() {
    let config = Config::read_config();
    println!("{:#?}", config);
    let mut client = Client::from_config(&config);

    let ts = client.fetch_daily("tsla", true);
    client.update_timeseries_raw(ts);

    /*let client = Client {
        key: String::from("asd"),
    };
    let ts = client.fetch_daily("TSLA", true);
    println!("{:#?}", ts);*/

}
