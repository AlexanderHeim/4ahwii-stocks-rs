use client::Client;
use config::Config;


mod config;
mod client;

fn main() {
    let config = Config::read_config();
    println!("{:#?}", config);
    let mut client = Client::from_config(&config);

    let mut ts = client.fetch_daily("tsla", false);
    Client::adjust_timeseries(&mut ts);
    println!("{:#?}", ts);

    /*let client = Client {
        key: String::from("asd"),
    };
    let ts = client.fetch_daily("TSLA", true);
    println!("{:#?}", ts);*/

}
