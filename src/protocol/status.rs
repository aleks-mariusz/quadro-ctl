use serde::Serialize;
use std::collections::BTreeMap;

use crate::config::FanLabel;

#[derive(Serialize)]
pub struct Status {
    pub device: DeviceInfo,
    pub temperatures: BTreeMap<String, Option<f64>>,
    pub fans: BTreeMap<FanLabel, FanStatus>,
    pub flow: f64,
}

#[derive(Serialize)]
pub struct DeviceInfo {
    pub serial: String,
    pub firmware: u16,
    pub power_cycles: u32,
}

#[derive(Serialize)]
pub struct FanStatus {
    pub rpm: u16,
    pub pwm: u16,
    pub voltage: f64,
    pub current: f64,
    pub power: f64,
}
