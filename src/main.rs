mod service;
mod socks5;
use clap::Parser;
use log;
use service::Service;
async fn run_main() {
    let mut service = Service::parse();
    log::debug!("代理地址为：{}:{}", service.host, service.port);
    service.run().await;
}

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    run_main().await;
}
