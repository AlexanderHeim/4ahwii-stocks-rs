use std::io::ErrorKind::NotFound;
use config::Config;

mod config;


fn main() {
    let mut config_file = match std::fs::File::open("config.toml") {
        Ok(config) => config,
        Err(error) => match error.kind() {
            NotFound => match std::fs::File::create("config.toml") {
                Ok(fc) => fc,
                Err(e) => panic!("Problem creating the config.toml file: {:?}", e),
            },
            other_error => panic!("Problem opening the config file: {:?}", other_error)
        }
    };


    let config = Config::parse_config(&mut config_file);
    println!("{:#?}", config);

    /*let client = Client {
        key: String::from("asd"),
    };
    let ts = client.fetch_daily("TSLA", true);
    println!("{:#?}", ts);*/

}
