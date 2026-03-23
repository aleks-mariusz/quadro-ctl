use crate::device::HidrawDevice;
use crate::error::QuadroError;

use super::{DeviceFactory, StandardLogger};

pub struct LinuxDeviceFactory;

impl DeviceFactory for LinuxDeviceFactory {
    fn open(&self, device_path: Option<&str>) -> Result<Box<dyn HidrawDevice>, QuadroError> {
        #[cfg(target_os = "linux")]
        {
            match device_path {
                Some(p) => Ok(Box::new(crate::device::LinuxHidrawDevice::open(
                    p,
                    Box::new(StandardLogger),
                )?)),
                None => Ok(Box::new(crate::device::find_quadro(
                    Box::new(StandardLogger),
                )?)),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = device_path;
            Err(QuadroError::UnsupportedPlatform)
        }
    }
}
