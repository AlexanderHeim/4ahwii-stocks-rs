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
    client.update_timeseries_raw(ts);
    client.get_raw_timeseries_between("tsla", Date::parse("2020-03-03", "%F").unwrap(), Date::parse("2021-01-01", "%F").unwrap());

    /*let client = Client {
        key: String::from("asd"),
    };
    let ts = client.fetch_daily("TSLA", true);
    println!("{:#?}", ts);*/

}
