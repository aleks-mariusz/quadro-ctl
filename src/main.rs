use clap::{Parser, Subcommand};

use quadro_ctl::config::QuadroConfig;
use quadro_ctl::services::{LinuxDeviceFactory, QuadroService, StandardLogger, ThreadSleeper};
use quadro_ctl::QuadroError;

#[derive(Parser)]
#[command(name = "quadro-ctl")]
#[command(about = "Bulk read/write for Aqua Computer QUADRO fan controller via hidraw")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Read {
        #[arg(long)]
        device: Option<String>,
    },
    Apply {
        #[arg(long)]
        device: Option<String>,
        #[arg(long)]
        config_file: String,
    },
    Status {
        #[arg(long)]
        device: Option<String>,
    },
}

fn main() -> Result<(), QuadroError> {
    let cli = Cli::parse();
    let service = QuadroService::new(LinuxDeviceFactory, StandardLogger, ThreadSleeper);

    match cli.command {
        Command::Read { device } => {
            let output = service.read(device.as_deref())?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
        Command::Apply {
            device,
            config_file,
        } => {
            let json_str = std::fs::read_to_string(&config_file)
                .map_err(|e| QuadroError::FileRead { path: config_file.clone(), source: e })?;
            let config: QuadroConfig = serde_json::from_str(&json_str)?;
            service.apply(device.as_deref(), &config)
        }
        Command::Status { device } => {
            let output = service.status(device.as_deref())?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
    }
}
