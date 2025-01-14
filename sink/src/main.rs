use std::env;

use anyhow::Error;
use clap::{Args, Parser};
use sink::{events::EventHandler, kg};
use substreams_utils::Sink;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const PKG_FILE: &str = "geo-substream.spkg";
const MODULE_NAME: &str = "geo_out";

const DEFAULT_START_BLOCK: i64 = 880;
const DEFAULT_END_BLOCK: u64 = 0;

#[tokio::main]
async fn main() -> Result<(), Error> {
    set_log_level();
    init_tracing();
    let endpoint_url =
        env::var("SUBSTREAMS_ENDPOINT_URL").expect("SUBSTREAMS_ENDPOINT_URL not set");
    let start_block = env::var("SUBSTREAMS_START_BLOCK")
        .ok()
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or_else(|| {
            tracing::warn!(
                "SUBSTREAMS_START_BLOCK not set. Using default value: {}",
                DEFAULT_START_BLOCK
            );
            DEFAULT_START_BLOCK
        });
    let end_block = env::var("SUBSTREAMS_END_BLOCK")
        .ok()
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or_else(|| {
            tracing::warn!(
                "SUBSTREAMS_END_BLOCK not set. Using default value: {}",
                DEFAULT_END_BLOCK
            );
            DEFAULT_END_BLOCK
        });

    let args = AppArgs::parse();

    let kg_client = kg::Client::new(
        &args.neo4j_args.neo4j_uri,
        &args.neo4j_args.neo4j_user,
        &args.neo4j_args.neo4j_pass,
    )
    .await?;

    if args.reset_db {
        kg_client.reset_db(args.rollup).await?;
    };

    let sink = EventHandler::new(kg_client);

    sink.run(&endpoint_url, PKG_FILE, MODULE_NAME, start_block, end_block)
        .await?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,

    /// Whether or not to roll up the relations
    #[arg(long, default_value_t = true)]
    rollup: bool,

    /// Whether or not to reset the database
    #[arg(long)]
    reset_db: bool,
}

#[derive(Debug, Args)]
struct Neo4jArgs {
    /// Neo4j database host
    #[arg(long)]
    neo4j_uri: String,

    /// Neo4j database user name
    #[arg(long)]
    neo4j_user: String,

    /// Neo4j database user password
    #[arg(long)]
    neo4j_pass: String,
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stdout=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn set_log_level() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
}
