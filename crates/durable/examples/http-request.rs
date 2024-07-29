use std::net::IpAddr;

use serde::Deserialize;

#[derive(Deserialize)]
struct Ip {
    origin: IpAddr,
}

fn main() {
    let response = durable::http::get("http://httpbin.org/ip")
        .send()
        .expect("failed to send HTTP request");
    let resp = response.json::<Ip>().expect("response was not valid UTF-8");

    println!("This workflow was run at {}", resp.origin);
}
