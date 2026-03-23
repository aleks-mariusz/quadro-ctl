use super::centi_percent::CentiPercent;
use super::millicelsius::Millicelsius;
use super::sensor_index::SensorIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurveData {
    pub sensor: SensorIndex,
    pub temps: [Millicelsius; 16],
    pub pwms: [CentiPercent; 16],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_data_stores_sensor_index() {
        let data = CurveData {
            sensor: SensorIndex::new(2).unwrap(),
            temps: [Millicelsius(0); 16],
            pwms: [CentiPercent(0); 16],
        };

        assert_eq!(data.sensor.value(), 2);
    }

    #[test]
    fn curve_data_stores_temperatures() {
        let mut temps = [Millicelsius(0); 16];
        temps[0] = Millicelsius(25000);
        let data = CurveData {
            sensor: SensorIndex::new(0).unwrap(),
            temps,
            pwms: [CentiPercent(0); 16],
        };

        assert_eq!(data.temps[0], Millicelsius(25000));
    }

    #[test]
    fn curve_data_stores_pwm_values() {
        let mut pwms = [CentiPercent(0); 16];
        pwms[0] = CentiPercent(5000);
        let data = CurveData {
            sensor: SensorIndex::new(0).unwrap(),
            temps: [Millicelsius(0); 16],
            pwms,
        };

        assert_eq!(data.pwms[0], CentiPercent(5000));
    }
}
