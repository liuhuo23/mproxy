use clap::Parser;
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(short = 'H', long, default_value = "127.0.0.1", help = "输入代理地址")]
    pub host: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 8080, help = "输入代理端口")]
    pub port: i32,
}
