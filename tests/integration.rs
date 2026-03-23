use quadro_ctl::config::{FanConfig, FanLabel, QuadroConfig};
use quadro_ctl::protocol::{
    self, CentiPercent, CurveData, FanId, Millicelsius, Percentage, Report, SensorIndex,
    CTRL_REPORT_ID, CTRL_REPORT_SIZE, SECONDARY_REPORT, SECONDARY_REPORT_ID,
};
use quadro_ctl::services::{MockDeviceFactory, NullLogger, NoopSleeper, QuadroService};

type TestService = QuadroService<MockDeviceFactory, NullLogger, NoopSleeper>;

fn parse_config(json: &str) -> QuadroConfig {
    serde_json::from_str(json).unwrap()
}

fn make_curve_points() -> Vec<serde_json::Value> {
    (0..16)
        .map(|i| {
            serde_json::json!({
                "temp": 20000 + i * 1000,
                "percentage": 20 + i * 5
            })
        })
        .collect()
}

fn mock_factory() -> MockDeviceFactory {
    MockDeviceFactory::new(vec![0u8; CTRL_REPORT_SIZE])
}

fn mock_factory_with(buffer: Vec<u8>) -> MockDeviceFactory {
    MockDeviceFactory::new(buffer)
}

fn test_service() -> TestService {
    QuadroService::new(mock_factory(), NullLogger, NoopSleeper)
}

fn test_service_with(factory: MockDeviceFactory) -> TestService {
    QuadroService::new(factory, NullLogger, NoopSleeper)
}

fn apply_manual_fan3_60() -> TestService {
    let service = test_service();
    let config = parse_config(r#"{"fans":{"fan3":{"mode":"manual","percentage":60}}}"#);
    service.apply(None, &config).unwrap();
    service
}

fn apply_curve_fan1_sensor2() -> TestService {
    let service = test_service();
    let points = make_curve_points();
    let json = serde_json::json!({
        "fans": {
            "fan1": {
                "mode": "curve",
                "sensor": 2,
                "points": points
            }
        }
    });
    let config: QuadroConfig = serde_json::from_value(json).unwrap();
    service.apply(None, &config).unwrap();
    service
}

fn apply_multi_fan_config() -> TestService {
    let service = test_service();
    let points = make_curve_points();
    let json = serde_json::json!({
        "fans": {
            "fan1": {
                "mode": "curve",
                "sensor": 1,
                "points": points
            },
            "fan3": {
                "mode": "manual",
                "percentage": 80
            }
        }
    });
    let config: QuadroConfig = serde_json::from_value(json).unwrap();
    service.apply(None, &config).unwrap();
    service
}

#[test]
fn apply_manual_sends_two_reports() {
    let service = apply_manual_fan3_60();
    let writes = service.device_factory().writes();

    assert_eq!(writes.len(), 2);
}

#[test]
fn apply_manual_first_report_has_ctrl_report_id() {
    let service = apply_manual_fan3_60();
    let writes = service.device_factory().writes();

    assert_eq!(writes[0].0, CTRL_REPORT_ID);
}

#[test]
fn apply_manual_sets_mode_byte_to_zero() {
    let service = apply_manual_fan3_60();
    let writes = service.device_factory().writes();
    let ref buffer = writes[0].1;
    let base = FanId::Fan3.offset();

    assert_eq!(buffer[base], 0x00);
}

#[test]
fn apply_manual_writes_correct_pwm_value() {
    let service = apply_manual_fan3_60();
    let writes = service.device_factory().writes();
    let ref buffer = writes[0].1;
    let base = FanId::Fan3.offset();

    assert_eq!(protocol::read_be16(buffer, base + 0x01), 6000);
}

#[test]
fn apply_curve_sets_mode_byte_to_one() {
    let service = apply_curve_fan1_sensor2();
    let writes = service.device_factory().writes();
    let ref buffer = writes[0].1;
    let base = FanId::Fan1.offset();

    assert_eq!(buffer[base], 0x02);
}

#[test]
fn apply_curve_writes_correct_sensor_id() {
    let service = apply_curve_fan1_sensor2();
    let writes = service.device_factory().writes();
    let ref buffer = writes[0].1;
    let base = FanId::Fan1.offset();

    assert_eq!(protocol::read_be16(buffer, base + 0x03), 2);
}

#[test]
fn apply_curve_writes_all_sixteen_temperature_and_pwm_points() {
    let service = apply_curve_fan1_sensor2();
    let writes = service.device_factory().writes();
    let ref buffer = writes[0].1;
    let base = FanId::Fan1.offset();

    for i in 0..16 {
        let expected_temp = 20000 + i as u16 * 1000;
        let expected_pwm = (20 + i * 5) as u16 * 100;
        assert_eq!(
            protocol::read_be16(buffer, base + 0x15 + i * 2),
            expected_temp
        );
        assert_eq!(
            protocol::read_be16(buffer, base + 0x35 + i * 2),
            expected_pwm
        );
    }
}

#[test]
fn apply_multi_fan_sets_fan1_curve_mode() {
    let service = apply_multi_fan_config();
    let writes = service.device_factory().writes();
    let ref buffer = writes[0].1;

    assert_eq!(buffer[FanId::Fan1.offset()], 0x02);
}

#[test]
fn apply_multi_fan_writes_fan1_sensor() {
    let service = apply_multi_fan_config();
    let writes = service.device_factory().writes();
    let ref buffer = writes[0].1;

    assert_eq!(protocol::read_be16(buffer, FanId::Fan1.offset() + 0x03), 1);
}

#[test]
fn apply_multi_fan_sets_fan3_manual_mode() {
    let service = apply_multi_fan_config();
    let writes = service.device_factory().writes();
    let ref buffer = writes[0].1;

    assert_eq!(buffer[FanId::Fan3.offset()], 0x00);
}

#[test]
fn apply_multi_fan_writes_fan3_pwm() {
    let service = apply_multi_fan_config();
    let writes = service.device_factory().writes();
    let ref buffer = writes[0].1;

    assert_eq!(protocol::read_be16(buffer, FanId::Fan3.offset() + 0x01), 8000);
}

#[test]
fn apply_preserves_unconfigured_fan_data() {
    let mut buffer = vec![0u8; CTRL_REPORT_SIZE];
    let fan2_base = FanId::Fan2.offset();
    buffer[fan2_base] = 0x02;
    protocol::write_be16(&mut buffer, fan2_base + 0x01, 4200);
    protocol::write_be16(&mut buffer, fan2_base + 0x03, 1);
    for i in 0..16 {
        protocol::write_be16(
            &mut buffer,
            fan2_base + 0x15 + i * 2,
            25000 + i as u16 * 500,
        );
        protocol::write_be16(
            &mut buffer,
            fan2_base + 0x35 + i * 2,
            (30 + i as u16) * 100,
        );
    }
    let fan2_snapshot: Vec<u8> = buffer[fan2_base..fan2_base + 0x55].to_vec();
    let config = parse_config(r#"{"fans":{"fan1":{"mode":"manual","percentage":40}}}"#);

    let service = test_service_with(mock_factory_with(buffer));
    service.apply(None, &config).unwrap();

    let writes = service.device_factory().writes();
    let ref written = writes[0].1;
    assert_eq!(&written[fan2_base..fan2_base + 0x55], &fan2_snapshot[..]);
}

#[test]
fn apply_produces_valid_checksum() {
    let config = parse_config(r#"{"fans":{"fan1":{"mode":"manual","percentage":50}}}"#);

    let service = test_service();
    service.apply(None, &config).unwrap();

    let writes = service.device_factory().writes();
    let ref buffer = writes[0].1;
    assert!(protocol::verify_checksum(buffer));
}

#[test]
fn apply_sends_secondary_report_with_correct_id() {
    let config = parse_config(r#"{"fans":{"fan1":{"mode":"manual","percentage":50}}}"#);

    let service = test_service();
    service.apply(None, &config).unwrap();

    let writes = service.device_factory().writes();
    assert_eq!(writes[1].0, SECONDARY_REPORT_ID);
}

#[test]
fn apply_sends_secondary_report_with_correct_payload() {
    let config = parse_config(r#"{"fans":{"fan1":{"mode":"manual","percentage":50}}}"#);

    let service = test_service();
    service.apply(None, &config).unwrap();

    let writes = service.device_factory().writes();
    assert_eq!(writes[1].1.as_slice(), &SECONDARY_REPORT);
}

fn read_device_with_manual_fan1_and_curve_fan2() -> Report {
    let mut buffer = vec![0u8; CTRL_REPORT_SIZE];
    protocol::apply_manual(
        &mut buffer,
        FanId::Fan1,
        CentiPercent::from_percentage(Percentage::new(50).unwrap()),
    );
    let mut temps = [Millicelsius(0); 16];
    let mut pwms = [CentiPercent(0); 16];
    for i in 0..16u16 {
        temps[i as usize] = Millicelsius(20000u16.wrapping_add(i * 1000));
        pwms[i as usize] = CentiPercent::from_percentage(Percentage::new((20 + i * 5) as u8).unwrap());
    }
    let curve_data = CurveData {
        sensor: SensorIndex::new(1).unwrap(),
        temps,
        pwms,
    };
    protocol::apply_curve(&mut buffer, FanId::Fan2, &curve_data);
    let service = test_service_with(mock_factory_with(buffer));
    service.read(None).unwrap()
}

#[test]
fn read_fan1_reports_manual_mode() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    assert!(matches!(report.fans[&FanLabel::Fan1], FanConfig::Manual { .. }));
}

#[test]
fn read_fan1_reports_correct_percentage() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    match &report.fans[&FanLabel::Fan1] {
        FanConfig::Manual { percentage } => assert_eq!(percentage.value(), 50),
        _ => panic!("expected manual"),
    }
}

#[test]
fn read_fan2_reports_curve_mode() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    assert!(matches!(report.fans[&FanLabel::Fan2], FanConfig::Curve { .. }));
}

#[test]
fn read_fan2_reports_correct_sensor() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    match &report.fans[&FanLabel::Fan2] {
        FanConfig::Curve { sensor, .. } => assert_eq!(sensor.value(), 1),
        _ => panic!("expected curve"),
    }
}

#[test]
fn read_fan2_has_sixteen_curve_points() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    match &report.fans[&FanLabel::Fan2] {
        FanConfig::Curve { points, .. } => assert_eq!(points.points().len(), 16),
        _ => panic!("expected curve"),
    }
}

#[test]
fn read_fan2_first_point_has_correct_temp() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    match &report.fans[&FanLabel::Fan2] {
        FanConfig::Curve { points, .. } => assert_eq!(points.points()[0].temp, 20000),
        _ => panic!("expected curve"),
    }
}

#[test]
fn read_fan2_first_point_has_correct_percentage() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    match &report.fans[&FanLabel::Fan2] {
        FanConfig::Curve { points, .. } => assert_eq!(points.points()[0].percentage.value(), 20),
        _ => panic!("expected curve"),
    }
}

#[test]
fn read_fan2_last_point_has_correct_temp() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    match &report.fans[&FanLabel::Fan2] {
        FanConfig::Curve { points, .. } => assert_eq!(points.points()[15].temp, 35000),
        _ => panic!("expected curve"),
    }
}

#[test]
fn read_fan2_last_point_has_correct_percentage() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    match &report.fans[&FanLabel::Fan2] {
        FanConfig::Curve { points, .. } => assert_eq!(points.points()[15].percentage.value(), 95),
        _ => panic!("expected curve"),
    }
}

#[test]
fn read_fan3_reports_manual_mode() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    assert!(matches!(report.fans[&FanLabel::Fan3], FanConfig::Manual { .. }));
}

#[test]
fn read_fan4_reports_manual_mode() {
    let report = read_device_with_manual_fan1_and_curve_fan2();

    assert!(matches!(report.fans[&FanLabel::Fan4], FanConfig::Manual { .. }));
}

fn apply_then_read_roundtrip() -> Report {
    let service = test_service();
    let points = make_curve_points();
    let json = serde_json::json!({
        "fans": {
            "fan1": {
                "mode": "curve",
                "sensor": 2,
                "points": points
            },
            "fan3": {
                "mode": "manual",
                "percentage": 75
            }
        }
    });
    let config: QuadroConfig = serde_json::from_value(json).unwrap();
    service.apply(None, &config).unwrap();
    let written_buffer = service.device_factory().writes()[0].1.clone();
    let read_service = test_service_with(mock_factory_with(written_buffer));
    read_service.read(None).unwrap()
}

#[test]
fn roundtrip_fan1_reads_back_as_curve_mode() {
    let report = apply_then_read_roundtrip();

    assert!(matches!(report.fans[&FanLabel::Fan1], FanConfig::Curve { .. }));
}

#[test]
fn roundtrip_fan1_reads_back_correct_sensor() {
    let report = apply_then_read_roundtrip();

    match &report.fans[&FanLabel::Fan1] {
        FanConfig::Curve { sensor, .. } => assert_eq!(sensor.value(), 2),
        _ => panic!("expected curve"),
    }
}

#[test]
fn roundtrip_fan1_reads_back_sixteen_points() {
    let report = apply_then_read_roundtrip();

    match &report.fans[&FanLabel::Fan1] {
        FanConfig::Curve { points, .. } => assert_eq!(points.points().len(), 16),
        _ => panic!("expected curve"),
    }
}

#[test]
fn roundtrip_fan1_reads_back_all_curve_point_values() {
    let report = apply_then_read_roundtrip();

    match &report.fans[&FanLabel::Fan1] {
        FanConfig::Curve { points, .. } => {
            for i in 0..16 {
                assert_eq!(points.points()[i].temp, 20000 + i as u16 * 1000);
                assert_eq!(points.points()[i].percentage.value(), (20 + i * 5) as u8);
            }
        }
        _ => panic!("expected curve"),
    }
}

#[test]
fn roundtrip_fan3_reads_back_as_manual_mode() {
    let report = apply_then_read_roundtrip();

    assert!(matches!(report.fans[&FanLabel::Fan3], FanConfig::Manual { .. }));
}

#[test]
fn roundtrip_fan3_reads_back_correct_percentage() {
    let report = apply_then_read_roundtrip();

    match &report.fans[&FanLabel::Fan3] {
        FanConfig::Manual { percentage } => assert_eq!(percentage.value(), 75),
        _ => panic!("expected manual"),
    }
}
