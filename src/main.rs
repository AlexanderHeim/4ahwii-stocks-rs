use rusty_stocks::Client;






fn main() {
    let client = Client {
        key: String::from("asd"),
    };
    let ts = client.fetch_daily("TSLA", true);
    println!("{:#?}", ts);

}
