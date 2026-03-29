mod service;
mod start;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = std::env::args().nth(1).map(std::path::PathBuf::from);
    Ok(start::run(config_path).await?)
}
