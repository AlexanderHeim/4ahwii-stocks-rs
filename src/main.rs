use client::Client;
use config::Config;
use mysql::time::Date;


mod config;
mod client;

fn main() {
    let config = Config::read_config();
    println!("{:#?}", config);
    let mut client = Client::from_config(&config);

    client.update("ibm");
    client.print_plot("ibm");

}
