use std::time::Duration;

use crate::config::QuadroConfig;
use crate::error::QuadroError;
use crate::protocol::{Report, Status};

use super::{DeviceFactory, Logger, Sleeper};

const CTRL_REPORT_DELAY: Duration = Duration::from_millis(200);

pub struct QuadroService<F: DeviceFactory, L: Logger, S: Sleeper> {
    device_factory: F,
    logger: L,
    sleeper: S,
}

impl<F: DeviceFactory, L: Logger, S: Sleeper> QuadroService<F, L, S> {
    pub fn new(device_factory: F, logger: L, sleeper: S) -> Self {
        Self { device_factory, logger, sleeper }
    }

    pub fn device_factory(&self) -> &F {
        &self.device_factory
    }

    pub fn read(&self, device_path: Option<&str>) -> Result<Report, QuadroError> {
        let mut device = self.device_factory.open(device_path)?;
        self.logger.info("[read] reading feature report");
        let raw = device.read_feature_report()?;

        if !raw.verify_checksum() {
            self.logger.error("checksum mismatch in feature report");
        }

        let report = raw.to_report();
        self.logger.info("[read] report parsed successfully");
        Ok(report)
    }

    pub fn apply(&self, device_path: Option<&str>, config: &QuadroConfig) -> Result<(), QuadroError> {
        let mut device = self.device_factory.open(device_path)?;
        self.logger.info("[apply] reading current feature report");
        let raw = device.read_feature_report()?;

        let report = raw.to_report();
        let updated = report.with_config(config);
        let updated_raw = raw.with_report(&updated);
        self.logger.info("[apply] config applied");

        self.sleeper.sleep(CTRL_REPORT_DELAY);
        device.write_feature_report(&updated_raw)?;
        self.logger.info("[apply] control report written");

        self.sleeper.sleep(CTRL_REPORT_DELAY);
        device.commit()?;
        self.logger.info("[apply] changes committed");

        Ok(())
    }

    pub fn status(&self, device_path: Option<&str>) -> Result<Status, QuadroError> {
        let mut device = self.device_factory.open(device_path)?;
        self.logger.info("[status] reading status report");
        let raw = device.read_status_report()?;
        let status = raw.to_status();
        self.logger.info("[status] report parsed successfully");
        Ok(status)
    }
}
