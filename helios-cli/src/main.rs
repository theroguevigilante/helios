use clap::{Parser, Subcommand};
use helios_parser::Registry;
use helios_parser_json::JsonParser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about = "Helios: Universal Log Pre-processing & Observability Platform", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Detect the format of a log file
    Detect {
        /// The path to the log file
        #[arg(short, long)]
        file: String,
    },
    /// Parse a log file and output normalized events
    Parse {
        /// The path to the log file
        #[arg(short, long)]
        file: String,
    },
    /// Parse and enrich log events
    Normalize {
        /// The path to the log file
        #[arg(short, long)]
        file: String,
    },
    /// Start the REST and WebSocket server
    Serve {
        /// The port to listen on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    /// Display statistics
    Stats,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();

    let mut registry = Registry::new();
    registry.register(JsonParser::new());

    match &cli.command {
        Commands::Detect { file } => {
            info!("Detecting format for file: {}", file);
        }
        Commands::Parse { file } => {
            info!("Parsing file: {}", file);
        }
        Commands::Normalize { file } => {
            info!("Normalizing file: {}", file);
        }
        Commands::Serve { port } => {
            info!("Starting server on port {}", port);
            helios_api::run_server(*port).await.unwrap();
        }
        Commands::Stats => {
            info!("Displaying stats");
        }
    }

    Ok(())
}
