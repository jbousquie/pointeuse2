use reqwest::blocking;

pub fn get_ohris_1() {
    let proxy = reqwest::Proxy::all("http://cache.iut-rodez.fr:8080/").unwrap();
    let client = blocking::Client::builder().proxy(proxy).cookie_store(true).build().unwrap();
    let resp = client.get("https://ohris.ut-capitole.fr/").send().unwrap();
    println!("{:?}", resp);
}
