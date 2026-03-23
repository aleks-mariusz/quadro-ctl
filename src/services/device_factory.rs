use crate::device::HidrawDevice;
use crate::error::QuadroError;

pub trait DeviceFactory {
    fn open(&self, device_path: Option<&str>) -> Result<Box<dyn HidrawDevice>, QuadroError>;
}
