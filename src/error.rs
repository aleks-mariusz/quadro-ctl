use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuadroError {
    #[error("failed to open device {path}: {source}")]
    DeviceOpen { path: String, source: std::io::Error },

    #[error("no Aquacomputer Quadro device found")]
    DeviceNotFound,

    #[error("ioctl {operation} failed: {source}")]
    Ioctl { operation: &'static str, source: std::io::Error },

    #[error("invalid device path: {0}")]
    InvalidDevicePath(String),

    #[error("failed to scan /dev: {0}")]
    DeviceScan(std::io::Error),

    #[error("buffer must not be empty")]
    EmptyBuffer,

    #[error("fan {fan}: {reason}")]
    InvalidConfig { fan: String, reason: String },

    #[error("failed to read feature report: {0}")]
    ReportRead(#[source] Box<QuadroError>),

    #[error("failed to write feature report: {0}")]
    ReportWrite(#[source] Box<QuadroError>),

    #[error("failed to read {path}: {source}")]
    FileRead { path: String, source: std::io::Error },

    #[error("failed to parse config: {0}")]
    ConfigParse(#[from] serde_json::Error),

    #[error("{field} value {value} out of range 0-{max}")]
    ValueOutOfRange { field: &'static str, value: u8, max: u8 },

    #[error("temperature {0}°C out of range 0.0-655.35")]
    TemperatureOutOfRange(f64),

    #[error("hidraw device access is only supported on Linux")]
    UnsupportedPlatform,
}
