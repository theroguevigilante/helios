use clap::{Parser, Subcommand};
use helios_detector::FormatDetector;
use helios_parser::Registry;
use helios_parser_cef::CefParser;
use helios_parser_json::JsonParser;
use helios_parser_syslog::SyslogParser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about = "Helios: Universal Log Pre-processing & Observability Platform", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Detect {
        #[arg(short, long)]
        file: String,
    },
    Parse {
        #[arg(short, long)]
        file: String,
    },
    Normalize {
        #[arg(short, long)]
        file: String,
    },
    Serve {
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    Stats,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();

    // Initialize Registry and register parsers
    let mut registry = Registry::new();
    registry.register(CefParser::new());
    registry.register(JsonParser::new());
    registry.register(SyslogParser::new());

    let detector = FormatDetector::new(&registry);

    match &cli.command {
        Commands::Detect { file } => {
            info!("Detecting format for file: {}", file);
        }
        Commands::Parse { file } => {
            info!("Parsing file: {}", file);
            let f = File::open(file)?;
            let reader = BufReader::new(f);

            for (line_num, line) in reader.lines().enumerate() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }

                // 1. Detect Format
                if let Some(parser_name) = detector.detect(&line) {
                    // 2. Find the parser and parse
                    if let Some(parser) =
                        registry.parsers().iter().find(|p| p.name() == parser_name)
                    {
                        match parser.parse(&line) {
                            Ok(event) => {
                                let json = serde_json::to_string_pretty(&event)?;
                                println!("Line {}: [{}] =>\n{}", line_num + 1, parser_name, json);
                            }
                            Err(e) => warn!("Failed to parse line {}: {}", line_num + 1, e),
                        }
                    }
                } else {
                    warn!(
                        "Line {}: Could not detect format for log: {}",
                        line_num + 1,
                        line
                    );
                }
            }
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
