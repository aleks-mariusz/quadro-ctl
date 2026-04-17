use super::buffer;
use super::constants::*;

pub struct RawVirtualSensorsReport(Vec<u8>);

impl RawVirtualSensorsReport {
    pub fn new(values: &[(usize, u16)]) -> Self {
        let mut buf = vec![0u8; VIRTUAL_SENSORS_REPORT_SIZE];
        buf[0] = VIRTUAL_SENSORS_REPORT_ID;

        for i in 0..16 {
            let offset = VIRTUAL_SENSORS_VALUES_OFFSET + i * SENSOR_SIZE;
            buffer::write_be16(&mut buf, offset, VIRTUAL_SENSOR_DISABLED_VALUE);
            buf[VIRTUAL_SENSORS_TYPES_OFFSET + i] = VIRTUAL_SENSOR_TYPE_DISABLED;
        }

        for &(index, centi_degrees) in values {
            let offset = VIRTUAL_SENSORS_VALUES_OFFSET + index * SENSOR_SIZE;
            buffer::write_be16(&mut buf, offset, centi_degrees);
            buf[VIRTUAL_SENSORS_TYPES_OFFSET + index] = VIRTUAL_SENSOR_TYPE_TEMPERATURE;
        }

        for i in 0..16 {
            buf[VIRTUAL_SENSORS_UNKNOWN_OFFSET + i] = VIRTUAL_SENSORS_UNKNOWN_BYTE;
        }

        buffer::finalize(&mut buf);
        Self(buf)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_report_id() {
        let report = RawVirtualSensorsReport::new(&[]);
        assert_eq!(report.as_bytes()[0], VIRTUAL_SENSORS_REPORT_ID);
    }

    #[test]
    fn new_has_correct_size() {
        let report = RawVirtualSensorsReport::new(&[]);
        assert_eq!(report.as_bytes().len(), VIRTUAL_SENSORS_REPORT_SIZE);
    }

    #[test]
    fn unspecified_sensors_are_disabled() {
        let report = RawVirtualSensorsReport::new(&[]);
        for i in 0..16 {
            let offset = VIRTUAL_SENSORS_VALUES_OFFSET + i * SENSOR_SIZE;
            assert_eq!(buffer::read_be16(report.as_bytes(), offset), VIRTUAL_SENSOR_DISABLED_VALUE);
            assert_eq!(report.as_bytes()[VIRTUAL_SENSORS_TYPES_OFFSET + i], VIRTUAL_SENSOR_TYPE_DISABLED);
        }
    }

    #[test]
    fn specified_sensor_has_temperature_type() {
        let report = RawVirtualSensorsReport::new(&[(0, 3000)]);
        let offset = VIRTUAL_SENSORS_VALUES_OFFSET;
        assert_eq!(buffer::read_be16(report.as_bytes(), offset), 3000);
        assert_eq!(report.as_bytes()[VIRTUAL_SENSORS_TYPES_OFFSET], VIRTUAL_SENSOR_TYPE_TEMPERATURE);
    }

    #[test]
    fn unknown_bytes_are_filled() {
        let report = RawVirtualSensorsReport::new(&[]);
        for i in 0..16 {
            assert_eq!(report.as_bytes()[VIRTUAL_SENSORS_UNKNOWN_OFFSET + i], VIRTUAL_SENSORS_UNKNOWN_BYTE);
        }
    }

    #[test]
    fn checksum_is_valid() {
        let report = RawVirtualSensorsReport::new(&[(2, 4500)]);
        assert!(buffer::verify_checksum(report.as_bytes()));
    }
}
