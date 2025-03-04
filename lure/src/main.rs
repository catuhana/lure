use lure::Command as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match lure::Cli::parse().subcommand {
        lure::Subcommands::Config(config) => config.run().await.map_err(Into::into),
        lure::Subcommands::Start(start) => start.run().await.map_err(Into::into),
    }
}
