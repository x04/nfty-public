use std::time::Instant;

use net::{
    request::{CronetRequest, UploadData},
    CronetEngine, EngineParams, Executor,
};

#[tokio::main]
async fn main() {
    let exec = Executor::new();
    let mut params = EngineParams::new();
    let engine = CronetEngine::new(&mut params);
    // let body = vec![];
    // let mut upload = UploadData::new(body);
    let mut req = CronetRequest::new(&engine, &exec);
    req.set_method("GET");
    req.set_header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4244.0 Safari/537.36");
    req.set_header("Cookie", "1=2");
    // req.set_body(&mut upload);

    loop {
        let response = req.start("https://httpbin.org/brotli").await;
        if let Some(err) = response.last_error {}
        unsafe {
            dbg!(std::str::from_utf8_unchecked(&response.body));
        }
    }
}
