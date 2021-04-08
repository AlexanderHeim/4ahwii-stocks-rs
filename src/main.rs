use config::Config;

mod config;


fn main() {
    


    let config = Config::read_config();
    println!("{:#?}", config);

    /*let client = Client {
        key: String::from("asd"),
    };
    let ts = client.fetch_daily("TSLA", true);
    println!("{:#?}", ts);*/

}
