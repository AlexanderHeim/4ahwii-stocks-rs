use rusty_stocks::Client;






fn main() {
    let client = Client {
        key: String::from("asd"),
    };
    client.large_fetch_daily("TSLA");

}
