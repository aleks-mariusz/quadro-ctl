use clap::{Parser, Subcommand};

use quadro_ctl::config::{QuadroConfig, VirtualSensorsConfig};
use quadro_ctl::services::{LinuxDeviceFactory, QuadroService, StandardLogger, ThreadSleeper};
use quadro_ctl::QuadroError;

#[derive(Parser)]
#[command(name = "quadro-ctl")]
#[command(about = "Bulk read/write for Aqua Computer QUADRO fan controller")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Fans {
        #[command(subcommand)]
        action: FansAction,
    },
    Sensors {
        #[command(subcommand)]
        action: SensorsAction,
    },
    Status {
        #[arg(long)]
        device: Option<String>,
    },
}

#[derive(Subcommand)]
enum FansAction {
    Get {
        #[arg(long)]
        device: Option<String>,
    },
    Set {
        #[arg(long)]
        device: Option<String>,
        #[arg(long)]
        config_file: String,
    },
}

#[derive(Subcommand)]
enum SensorsAction {
    Set {
        #[arg(long)]
        device: Option<String>,
        #[arg(long)]
        config_file: String,
    },
}

fn main() -> Result<(), QuadroError> {
    let cli = Cli::parse();
    let service = QuadroService::new(LinuxDeviceFactory, StandardLogger, ThreadSleeper);

    match cli.command {
        Command::Fans { action } => match action {
            FansAction::Get { device } => {
                let output = service.read(device.as_deref())?;
                println!("{}", serde_json::to_string_pretty(&output)?);
                Ok(())
            }
            FansAction::Set { device, config_file } => {
                let json_str = std::fs::read_to_string(&config_file)
                    .map_err(|e| QuadroError::FileRead { path: config_file.clone(), source: e })?;
                let config: QuadroConfig = serde_json::from_str(&json_str)?;
                service.apply(device.as_deref(), &config)
            }
        },
        Command::Sensors { action } => match action {
            SensorsAction::Set { device, config_file } => {
                let json_str = std::fs::read_to_string(&config_file)
                    .map_err(|e| QuadroError::FileRead { path: config_file.clone(), source: e })?;
                let config: VirtualSensorsConfig = serde_json::from_str(&json_str)?;
                service.set_virtual_sensors(device.as_deref(), &config)
            }
        },
        Command::Status { device } => {
            let output = service.status(device.as_deref())?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
    }
}
