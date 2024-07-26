fn main() {
    let response = durable::http::get("http://httpbin.org/ip")
        .send()
        .expect("failed to send HTTP request");
    let text = response.text().expect("response was not valid UTF-8");

    durable::print(text);
}

durable::durable_main!(main);
