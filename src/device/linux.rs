use std::os::unix::io::RawFd;

use crate::error::QuadroError;
use crate::protocol::{RawReport, RawStatusReport, CTRL_REPORT_ID, CTRL_REPORT_SIZE, SECONDARY_REPORT, SECONDARY_REPORT_ID, STATUS_REPORT_SIZE};
use crate::services::Logger;
use super::HidrawDevice;

const IOC_WRITE: u32 = 1;
const IOC_READ: u32 = 2;

fn ioctl_ioc(dir: u32, typ: u32, nr: u32, size: u32) -> libc::c_ulong {
    ((dir << 30) | (size << 16) | (typ << 8) | nr) as libc::c_ulong
}

fn hidioc_gfeature(len: usize) -> libc::c_ulong {
    ioctl_ioc(IOC_WRITE | IOC_READ, b'H' as u32, 0x07, len as u32)
}

fn hidioc_sfeature(len: usize) -> libc::c_ulong {
    ioctl_ioc(IOC_WRITE | IOC_READ, b'H' as u32, 0x06, len as u32)
}

fn hidioc_grawinfo() -> libc::c_ulong {
    ioctl_ioc(IOC_READ, b'H' as u32, 0x03, std::mem::size_of::<HidrawDevinfo>() as u32)
}

#[repr(C)]
struct HidrawDevinfo {
    bustype: u32,
    vendor: i16,
    product: i16,
}

const QUADRO_VENDOR: i16 = 0x0c70;
const QUADRO_PRODUCT: i16 = 0xf00d_u16 as i16;

pub struct LinuxHidrawDevice {
    fd: RawFd,
    logger: Box<dyn Logger>,
}

impl LinuxHidrawDevice {
    pub fn open(path: &str, logger: Box<dyn Logger>) -> Result<Self, QuadroError> {
        logger.info(&format!("[device] opening {}", path));
        let c_path =
            std::ffi::CString::new(path).map_err(|e| QuadroError::InvalidDevicePath(e.to_string()))?;
        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR) };
        if fd < 0 {
            let err = std::io::Error::last_os_error();
            logger.error(&format!("[device] failed to open {}: {}", path, err));
            return Err(QuadroError::DeviceOpen { path: path.to_string(), source: err });
        }
        logger.info(&format!("[device] opened {} -> fd={}", path, fd));
        Ok(Self { fd, logger })
    }

    fn ioctl_read(&self, report_id: u8, size: usize) -> Result<Vec<u8>, QuadroError> {
        let mut buf = vec![0u8; size];
        buf[0] = report_id;
        let ioctl_num = hidioc_gfeature(size);
        self.logger.info(&format!(
            "[device] HIDIOCGFEATURE: fd={}, report_id=0x{:02x}, buf_len={}",
            self.fd, report_id, size
        ));
        let ret = unsafe { libc::ioctl(self.fd, ioctl_num, buf.as_mut_ptr()) };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            self.logger.error(&format!("[device] HIDIOCGFEATURE error: {}", err));
            return Err(QuadroError::Ioctl { operation: "HIDIOCGFEATURE", source: err });
        }
        self.logger.info(&format!("[device] HIDIOCGFEATURE read {} bytes", ret));
        Ok(buf)
    }

    fn ioctl_write(&self, report_id: u8, data: &[u8]) -> Result<usize, QuadroError> {
        let mut buf = data.to_vec();
        buf[0] = report_id;
        let ioctl_num = hidioc_sfeature(buf.len());
        self.logger.info(&format!(
            "[device] HIDIOCSFEATURE: fd={}, report_id=0x{:02x}, buf_len={}",
            self.fd, report_id, buf.len()
        ));
        let ret = unsafe { libc::ioctl(self.fd, ioctl_num, buf.as_ptr()) };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            self.logger.error(&format!("[device] HIDIOCSFEATURE error: {}", err));
            return Err(QuadroError::Ioctl { operation: "HIDIOCSFEATURE", source: err });
        }
        self.logger.info(&format!("[device] HIDIOCSFEATURE wrote {} bytes", ret));
        Ok(ret as usize)
    }
}

impl HidrawDevice for LinuxHidrawDevice {
    fn read_feature_report(&mut self) -> Result<RawReport, QuadroError> {
        let buf = self.ioctl_read(CTRL_REPORT_ID, CTRL_REPORT_SIZE)
            .map_err(|e| QuadroError::ReportRead(Box::new(e)))?;
        Ok(RawReport::from_bytes(buf))
    }

    fn write_feature_report(&mut self, report: &RawReport) -> Result<(), QuadroError> {
        self.ioctl_write(CTRL_REPORT_ID, report.as_bytes())
            .map_err(|e| QuadroError::ReportWrite(Box::new(e)))?;
        Ok(())
    }

    fn commit(&mut self) -> Result<(), QuadroError> {
        self.ioctl_write(SECONDARY_REPORT_ID, &SECONDARY_REPORT)
            .map_err(|e| QuadroError::ReportWrite(Box::new(e)))?;
        Ok(())
    }

    fn read_status_report(&mut self) -> Result<RawStatusReport, QuadroError> {
        let mut buf = vec![0u8; STATUS_REPORT_SIZE];
        self.logger.info(&format!(
            "[device] reading status report ({} bytes)",
            STATUS_REPORT_SIZE
        ));
        let ret = unsafe {
            libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            self.logger.error(&format!("[device] read error: {}", err));
            return Err(QuadroError::ReportRead(Box::new(
                QuadroError::Ioctl { operation: "read", source: err }
            )));
        }
        self.logger.info(&format!("[device] read {} bytes", ret));
        Ok(RawStatusReport::from_bytes(buf))
    }
}

impl Drop for LinuxHidrawDevice {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd); }
    }
}

pub fn find_quadro(logger: Box<dyn Logger>) -> Result<LinuxHidrawDevice, QuadroError> {
    logger.info(&format!(
        "[device] scanning /dev/hidraw* for QUADRO (vendor=0x{:04x}, product=0x{:04x})",
        QUADRO_VENDOR as u16, QUADRO_PRODUCT as u16
    ));
    let entries = std::fs::read_dir("/dev").map_err(QuadroError::DeviceScan)?;

    for entry in entries {
        let entry = entry.map_err(QuadroError::DeviceScan)?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("hidraw") {
            continue;
        }

        let path = entry.path();
        let path_str = path
            .to_str()
            .ok_or_else(|| QuadroError::InvalidDevicePath("non-utf8 hidraw path".into()))?;

        let device = match LinuxHidrawDevice::open(path_str, Box::new(crate::NullLogger)) {
            Ok(d) => d,
            Err(e) => {
                logger.info(&format!("[device] skip {}: {}", path_str, e));
                continue;
            }
        };

        let mut info: HidrawDevinfo = unsafe { std::mem::zeroed() };
        let ret = unsafe {
            libc::ioctl(device.fd, hidioc_grawinfo(), &mut info as *mut HidrawDevinfo)
        };
        if ret < 0 {
            logger.info(&format!("[device] skip {}: HIDIOCGRAWINFO failed", path_str));
            continue;
        }

        logger.info(&format!(
            "[device] {} -> vendor=0x{:04x} product=0x{:04x}",
            path_str, info.vendor as u16, info.product as u16
        ));

        if info.vendor == QUADRO_VENDOR && info.product == QUADRO_PRODUCT {
            logger.info(&format!("[device] found QUADRO at {}", path_str));
            drop(device);
            return LinuxHidrawDevice::open(path_str, logger);
        }
    }

    Err(QuadroError::DeviceNotFound)
}
