use clap::{Parser, Subcommand};
use helios_detector::FormatDetector;
use helios_ingest::{EvtxFileSource, LogSource};
use helios_parser::Registry;
use helios_parser_android::AndroidParser;
use helios_parser_apache::ApacheParser;
use helios_parser_cef::CefParser;
use helios_parser_evtx::EvtxParser;
use helios_parser_json::JsonParser;
use helios_parser_nginx::NginxParser;
use helios_parser_openssh::OpenSshParser;
use helios_parser_proxifier::ProxifierParser;
use helios_parser_spark::SparkParser;
use helios_parser_syslog::SyslogParser;
use helios_parser_windows::WindowsParser;
use helios_parser_zookeeper::ZooKeeperParser;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use tokio::sync::mpsc;
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
    registry.register(ApacheParser::new());
    registry.register(NginxParser::new());
    registry.register(JsonParser::new());
    registry.register(EvtxParser::new());
    registry.register(OpenSshParser::new());
    registry.register(SyslogParser::new());
    registry.register(ZooKeeperParser::new());
    registry.register(SparkParser::new());
    registry.register(WindowsParser::new());
    registry.register(AndroidParser::new());
    registry.register(ProxifierParser::new());

    let detector = FormatDetector::new(&registry);

    match &cli.command {
        Commands::Detect { file } => {
            info!("Detecting format for file: {}", file);
            let mut f = File::open(file)?;
            let mut magic = [0u8; 8];
            let is_evtx = if f.read_exact(&mut magic).is_ok() && &magic == b"ElfFile\0" {
                true
            } else {
                file.ends_with(".evtx")
            };

            if is_evtx {
                info!("Detected EVTX binary file. Using EvtxFileSource...");
                let source = EvtxFileSource::new(file.clone());
                let (tx, mut rx) = mpsc::channel(100);

                tokio::spawn(async move {
                    if let Err(e) = source.run(tx).await {
                        warn!("EVTX source error: {}", e);
                    }
                });

                let mut record_num = 0;
                while let Some(line) = rx.recv().await {
                    record_num += 1;
                    if let Some(parser_name) = detector.detect(&line) {
                        println!("Record {}: [{}]", record_num, parser_name);
                    } else {
                        println!("Record {}: [UNKNOWN]", record_num);
                    }
                }
            } else {
                use std::io::Seek;
                f.seek(std::io::SeekFrom::Start(0))?;
                let reader = BufReader::new(f);

                for (line_num, line) in reader.lines().enumerate() {
                    let line = line?;
                    if line.trim().is_empty() {
                        continue;
                    }

                    if let Some(parser_name) = detector.detect(&line) {
                        println!("Line {}: [{}]", line_num + 1, parser_name);
                    } else {
                        println!("Line {}: [UNKNOWN]", line_num + 1);
                    }
                }
            }
        }
        Commands::Parse { file } => {
            info!("Parsing file: {}", file);
            let mut f = File::open(file)?;
            let mut magic = [0u8; 8];
            let is_evtx = if f.read_exact(&mut magic).is_ok() && &magic == b"ElfFile\0" {
                true
            } else {
                file.ends_with(".evtx")
            };

            if is_evtx {
                info!("Detected EVTX binary file. Using EvtxFileSource...");
                let source = EvtxFileSource::new(file.clone());
                let (tx, mut rx) = mpsc::channel(100);

                tokio::spawn(async move {
                    if let Err(e) = source.run(tx).await {
                        warn!("EVTX source error: {}", e);
                    }
                });

                let mut record_num = 0;
                while let Some(line) = rx.recv().await {
                    record_num += 1;
                    if let Some(parser_name) = detector.detect(&line) {
                        if let Some(parser) =
                            registry.parsers().iter().find(|p| p.name() == parser_name)
                        {
                            match parser.parse(&line) {
                                Ok(event) => {
                                    let json = serde_json::to_string_pretty(&event)?;
                                    println!(
                                        "Record {}: [{}] =>\n{}",
                                        record_num, parser_name, json
                                    );
                                }
                                Err(e) => warn!("Failed to parse record {}: {}", record_num, e),
                            }
                        }
                    } else {
                        warn!(
                            "Record {}: Could not detect format for EVTX JSON",
                            record_num
                        );
                    }
                }
            } else {
                // Rewind file for normal parsing
                use std::io::Seek;
                f.seek(std::io::SeekFrom::Start(0))?;
                let reader = BufReader::new(f);

                for (line_num, line) in reader.lines().enumerate() {
                    let line = line?;
                    if line.trim().is_empty() {
                        continue;
                    }

                    if let Some(parser_name) = detector.detect(&line) {
                        if let Some(parser) =
                            registry.parsers().iter().find(|p| p.name() == parser_name)
                        {
                            match parser.parse(&line) {
                                Ok(event) => {
                                    let json = serde_json::to_string_pretty(&event)?;
                                    println!(
                                        "Line {}: [{}] =>\n{}",
                                        line_num + 1,
                                        parser_name,
                                        json
                                    );
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
