use rusty_stocks::Client;






fn main() {
    let client = Client {
        key: String::from("key"),
    };
    client.large_fetch_daily("ibm");

}
