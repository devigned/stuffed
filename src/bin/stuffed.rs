use clap::Parser;
use std::process::exit;
use tracing_subscriber::EnvFilter;
use stuffed::commands::{PullCommand, PushCommand};

fn version() -> &'static str {
    option_env!("CARGO_VERSION_INFO").unwrap_or(env!("CARGO_PKG_VERSION"))
}

/// Warg component registry client.
#[derive(Parser)]
#[clap(
    bin_name = "stuffed",
    version,
    propagate_version = true,
    arg_required_else_help = true
)]
#[command(version = version())]
enum StuffedCli {
    Push(PushCommand),
    Pull(PullCommand)
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    if let Err(e) = match StuffedCli::parse() {
        StuffedCli::Push(cmd) => cmd.exec().await,
        StuffedCli::Pull(cmd) => cmd.exec().await,
    } {
        eprintln!("error: {e:?}");
        exit(1);
    }

    Ok(())
}