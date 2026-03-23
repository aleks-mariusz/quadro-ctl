use std::collections::BTreeMap;

use crate::config::{Curve, CurvePoint, FanConfig, FanLabel};

use super::buffer;
use super::centi_percent::CentiPercent;
use super::curve_data::CurveData;
use super::fan::{FanId, FanMode};
use super::millicelsius::Millicelsius;
use super::report::Report;

pub struct RawReport(Vec<u8>);

const FAN_LABELS: [(FanLabel, FanId); 4] = [
    (FanLabel::Fan1, FanId::Fan1),
    (FanLabel::Fan2, FanId::Fan2),
    (FanLabel::Fan3, FanId::Fan3),
    (FanLabel::Fan4, FanId::Fan4),
];

impl RawReport {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn verify_checksum(&self) -> bool {
        buffer::verify_checksum(&self.0)
    }

    pub fn to_report(&self) -> Report {
        let mut fans = BTreeMap::new();
        for (label, fan_id) in &FAN_LABELS {
            let mode = buffer::read_fan_mode(&self.0, *fan_id);
            let fan_config = match mode {
                FanMode::Manual => {
                    let pwm = buffer::read_manual_pwm(&self.0, *fan_id);
                    FanConfig::Manual {
                        percentage: pwm.to_percentage(),
                    }
                }
                FanMode::Curve => {
                    let curve_data = buffer::read_curve(&self.0, *fan_id);
                    let points: Vec<CurvePoint> = curve_data
                        .temps
                        .iter()
                        .zip(curve_data.pwms.iter())
                        .map(|(t, p)| CurvePoint {
                            temp: t.0,
                            percentage: p.to_percentage(),
                        })
                        .collect();
                    FanConfig::Curve {
                        sensor: curve_data.sensor,
                        points: Curve::new(points).expect("device returned invalid curve"),
                    }
                }
            };
            fans.insert(*label, fan_config);
        }
        Report { fans }
    }

    pub fn with_report(&self, report: &Report) -> RawReport {
        let mut buf = self.0.clone();

        for (label, fan_id) in &FAN_LABELS {
            if let Some(fan_config) = report.fans.get(label) {
                match fan_config {
                    FanConfig::Manual { percentage } => {
                        let cp = CentiPercent::from_percentage(*percentage);
                        buffer::apply_manual(&mut buf, *fan_id, cp);
                    }
                    FanConfig::Curve { sensor, points } => {
                        let mut temps = [Millicelsius(0); 16];
                        let mut pwms = [CentiPercent(0); 16];
                        for (i, point) in points.points().iter().enumerate() {
                            temps[i] = Millicelsius(point.temp);
                            pwms[i] = CentiPercent::from_percentage(point.percentage);
                        }
                        let curve_data = CurveData {
                            sensor: *sensor,
                            temps,
                            pwms,
                        };
                        buffer::apply_curve(&mut buf, *fan_id, &curve_data);
                    }
                }
            }
        }

        buffer::finalize(&mut buf);
        RawReport(buf)
    }
}
