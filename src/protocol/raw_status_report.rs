use std::collections::BTreeMap;

use crate::config::FanLabel;

use super::buffer;
use super::constants::*;
use super::status::{DeviceInfo, FanStatus, Status};

const DISCONNECTED_SENSOR: i16 = 0x7FFF;

pub struct RawStatusReport(Vec<u8>);

impl RawStatusReport {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn to_status(&self) -> Status {
        Status {
            device: self.parse_device_info(),
            temperatures: self.parse_temperatures(),
            fans: self.parse_fans(),
            flow: self.parse_flow(),
        }
    }

    fn parse_device_info(&self) -> DeviceInfo {
        let serial_part1 = buffer::read_be16(&self.0, AQC_SERIAL_START);
        let serial_part2 = buffer::read_be16(&self.0, AQC_SERIAL_START + 2);
        let serial = format!("{:05}-{:05}", serial_part1, serial_part2);
        let firmware = buffer::read_be16(&self.0, AQC_FIRMWARE_VERSION);
        let power_cycles = u32::from_be_bytes([
            self.0[AQC_POWER_CYCLES],
            self.0[AQC_POWER_CYCLES + 1],
            self.0[AQC_POWER_CYCLES + 2],
            self.0[AQC_POWER_CYCLES + 3],
        ]);
        DeviceInfo { serial, firmware, power_cycles }
    }

    fn parse_temperatures(&self) -> BTreeMap<String, Option<f64>> {
        let mut temps = BTreeMap::new();
        for i in 0..QUADRO_NUM_SENSORS {
            let offset = QUADRO_SENSOR_START + i * SENSOR_SIZE;
            let raw = buffer::read_be16(&self.0, offset) as i16;
            let value = if raw == DISCONNECTED_SENSOR {
                None
            } else {
                Some(raw as f64 / 100.0)
            };
            temps.insert(format!("sensor{}", i + 1), value);
        }
        temps
    }

    fn parse_fans(&self) -> BTreeMap<FanLabel, FanStatus> {
        let labels = [FanLabel::Fan1, FanLabel::Fan2, FanLabel::Fan3, FanLabel::Fan4];
        labels.iter().enumerate().map(|(i, label)| {
            let base = QUADRO_FAN_SENSOR_OFFSETS[i];
            let fan = FanStatus {
                pwm: buffer::read_be16(&self.0, base),
                voltage: buffer::read_be16(&self.0, base + AQC_FAN_VOLTAGE_OFFSET) as f64 / 100.0,
                current: buffer::read_be16(&self.0, base + AQC_FAN_CURRENT_OFFSET) as f64 / 100.0,
                power: buffer::read_be16(&self.0, base + AQC_FAN_POWER_OFFSET) as f64 / 100.0,
                rpm: buffer::read_be16(&self.0, base + AQC_FAN_SPEED_OFFSET),
            };
            (*label, fan)
        }).collect()
    }

    fn parse_flow(&self) -> f64 {
        buffer::read_be16(&self.0, QUADRO_FLOW_SENSOR_OFFSET) as f64 / 10.0
    }
}
