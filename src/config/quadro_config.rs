use serde::Deserialize;
use std::collections::HashMap;

use super::fan_config::{FanConfig, FanLabel};

#[derive(Debug, Clone, Deserialize)]
pub struct QuadroConfig {
    pub fans: HashMap<FanLabel, FanConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_manual_config() -> QuadroConfig {
        serde_json::from_str(r#"{"fans":{"fan1":{"mode":"manual","percentage":50}}}"#).unwrap()
    }

    fn parse_curve_config() -> QuadroConfig {
        let points: Vec<serde_json::Value> = (0..16)
            .map(|i| serde_json::json!({"temp": 20.0 + i as f64 * 2.0, "percentage": 20 + i * 5}))
            .collect();
        let json = serde_json::json!({
            "fans": { "fan2": { "mode": "curve", "sensor": 0, "points": points } }
        });
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn manual_fan_json_deserializes_with_one_entry() {
        let config = parse_manual_config();

        assert_eq!(config.fans.len(), 1);
    }

    #[test]
    fn manual_fan_json_contains_fan1_key() {
        let config = parse_manual_config();

        assert!(config.fans.contains_key(&FanLabel::Fan1));
    }

    #[test]
    fn manual_fan_json_has_correct_percentage() {
        let config = parse_manual_config();

        match &config.fans[&FanLabel::Fan1] {
            FanConfig::Manual { percentage } => assert_eq!(percentage.value(), 50),
            _ => panic!("expected Manual"),
        }
    }

    #[test]
    fn curve_fan_json_deserializes_with_one_entry() {
        let config = parse_curve_config();

        assert_eq!(config.fans.len(), 1);
    }

    #[test]
    fn curve_fan_json_has_correct_sensor() {
        let config = parse_curve_config();

        match &config.fans[&FanLabel::Fan2] {
            FanConfig::Curve { sensor, .. } => assert_eq!(sensor.value(), 0),
            _ => panic!("expected Curve"),
        }
    }

    #[test]
    fn curve_fan_json_has_sixteen_points() {
        let config = parse_curve_config();

        match &config.fans[&FanLabel::Fan2] {
            FanConfig::Curve { points, .. } => assert_eq!(points.points().len(), 16),
            _ => panic!("expected Curve"),
        }
    }

    #[test]
    fn multiple_fans_deserialize_in_single_config() {
        let points: Vec<serde_json::Value> = (0..16)
            .map(|i| serde_json::json!({"temp": 20.0 + i as f64 * 2.0, "percentage": 20 + i * 5}))
            .collect();
        let json = serde_json::json!({
            "fans": {
                "fan1": { "mode": "manual", "percentage": 75 },
                "fan3": { "mode": "curve", "sensor": 1, "points": points }
            }
        });

        let config: QuadroConfig = serde_json::from_value(json).unwrap();

        assert_eq!(config.fans.len(), 2);
    }

    #[test]
    fn percentage_above_100_fails_deserialization() {
        let json = r#"{"fans":{"fan1":{"mode":"manual","percentage":101}}}"#;

        let result: Result<QuadroConfig, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }

    #[test]
    fn sensor_index_above_19_fails_deserialization() {
        let points: Vec<serde_json::Value> = (0..16)
            .map(|i| serde_json::json!({"temp": 20.0 + i as f64 * 2.0, "percentage": 20 + i * 5}))
            .collect();
        let json = serde_json::json!({
            "fans": {"fan2": {"mode": "curve", "sensor": 20, "points": points}}
        });

        let result: Result<QuadroConfig, _> = serde_json::from_value(json);

        assert!(result.is_err());
    }

    #[test]
    fn virtual_sensor_index_4_deserializes() {
        let points: Vec<serde_json::Value> = (0..16)
            .map(|i| serde_json::json!({"temp": 20.0 + i as f64 * 2.0, "percentage": 20 + i * 5}))
            .collect();
        let json = serde_json::json!({
            "fans": {"fan2": {"mode": "curve", "sensor": 4, "points": points}}
        });

        let config: QuadroConfig = serde_json::from_value(json).unwrap();

        match &config.fans[&FanLabel::Fan2] {
            FanConfig::Curve { sensor, .. } => assert_eq!(sensor.value(), 4),
            _ => panic!("expected Curve"),
        }
    }

    #[test]
    fn curve_with_wrong_point_count_fails_deserialization() {
        let points: Vec<serde_json::Value> = (0..8)
            .map(|i| serde_json::json!({"temp": 20.0 + i as f64 * 2.0, "percentage": 20 + i * 5}))
            .collect();
        let json = serde_json::json!({
            "fans": {"fan2": {"mode": "curve", "sensor": 0, "points": points}}
        });

        let result: Result<QuadroConfig, _> = serde_json::from_value(json);

        assert!(result.is_err());
    }

    #[test]
    fn non_monotonic_temperatures_fail_deserialization() {
        let mut points: Vec<serde_json::Value> = (0..16)
            .map(|i| serde_json::json!({"temp": 20.0 + i as f64 * 2.0, "percentage": 20 + i * 5}))
            .collect();
        points[5] = serde_json::json!({"temp": 20.0, "percentage": 40});
        let json = serde_json::json!({
            "fans": {"fan2": {"mode": "curve", "sensor": 0, "points": points}}
        });

        let result: Result<QuadroConfig, _> = serde_json::from_value(json);

        assert!(result.is_err());
    }
}
