use crate::error::QuadroError;
use super::fan_config::CurvePoint;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Curve(Vec<CurvePoint>);

impl Curve {
    pub fn new(points: Vec<CurvePoint>) -> Result<Self, QuadroError> {
        if points.len() != 16 {
            return Err(QuadroError::InvalidConfig {
                fan: String::new(),
                reason: format!(
                    "curve must have exactly 16 points, got {}",
                    points.len()
                ),
            });
        }
        for window in points.windows(2) {
            if window[1].temp <= window[0].temp {
                return Err(QuadroError::InvalidConfig {
                    fan: String::new(),
                    reason: format!(
                        "temperatures must be monotonically increasing, found {} followed by {}",
                        window[0].temp, window[1].temp
                    ),
                });
            }
        }
        Ok(Self(points))
    }

    pub fn points(&self) -> &[CurvePoint] {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for Curve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let points = Vec::<CurvePoint>::deserialize(deserializer)?;
        Curve::new(points).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Percentage;

    fn valid_points() -> Vec<CurvePoint> {
        (0..16)
            .map(|i| CurvePoint {
                temp: 20000 + i * 2000,
                percentage: Percentage::new((20 + i * 5) as u8).unwrap(),
            })
            .collect()
    }

    #[test]
    fn sixteen_monotonic_points_are_valid() {
        assert!(Curve::new(valid_points()).is_ok());
    }

    #[test]
    fn fewer_than_sixteen_points_rejected() {
        let points: Vec<CurvePoint> = valid_points().into_iter().take(8).collect();
        assert!(Curve::new(points).is_err());
    }

    #[test]
    fn more_than_sixteen_points_rejected() {
        let mut points = valid_points();
        points.push(CurvePoint {
            temp: 60000,
            percentage: Percentage::new(99).unwrap(),
        });
        assert!(Curve::new(points).is_err());
    }

    #[test]
    fn equal_temperatures_rejected() {
        let mut points = valid_points();
        points[5].temp = points[4].temp;
        assert!(Curve::new(points).is_err());
    }

    #[test]
    fn decreasing_temperatures_rejected() {
        let mut points = valid_points();
        points[5].temp = points[4].temp - 1;
        assert!(Curve::new(points).is_err());
    }

    #[test]
    fn points_accessor_returns_all_sixteen() {
        let curve = Curve::new(valid_points()).unwrap();
        assert_eq!(curve.points().len(), 16);
    }

    #[test]
    fn deserialize_valid_curve() {
        let json: Vec<serde_json::Value> = (0..16)
            .map(|i| serde_json::json!({"temp": 20000 + i * 2000, "percentage": 20 + i * 5}))
            .collect();
        let json_str = serde_json::to_string(&json).unwrap();
        let curve: Curve = serde_json::from_str(&json_str).unwrap();
        assert_eq!(curve.points().len(), 16);
    }

    #[test]
    fn deserialize_wrong_count_fails() {
        let json: Vec<serde_json::Value> = (0..8)
            .map(|i| serde_json::json!({"temp": 20000 + i * 2000, "percentage": 20 + i * 5}))
            .collect();
        let json_str = serde_json::to_string(&json).unwrap();
        let result: Result<Curve, _> = serde_json::from_str(&json_str);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_non_monotonic_fails() {
        let mut json: Vec<serde_json::Value> = (0..16)
            .map(|i| serde_json::json!({"temp": 20000 + i * 2000, "percentage": 20 + i * 5}))
            .collect();
        json[5] = serde_json::json!({"temp": 20000, "percentage": 40});
        let json_str = serde_json::to_string(&json).unwrap();
        let result: Result<Curve, _> = serde_json::from_str(&json_str);
        assert!(result.is_err());
    }
}
