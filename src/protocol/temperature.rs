use crate::error::QuadroError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Temperature(u16);

impl Temperature {
    pub fn from_celsius(celsius: f64) -> Result<Self, QuadroError> {
        if celsius < 0.0 || celsius > 655.35 {
            return Err(QuadroError::TemperatureOutOfRange(celsius));
        }
        Ok(Self((celsius * 100.0).round() as u16))
    }

    pub fn from_centi_degrees(centi_degrees: u16) -> Self {
        Self(centi_degrees)
    }

    pub fn to_celsius(self) -> f64 {
        self.0 as f64 / 100.0
    }

    pub fn to_centi_degrees(self) -> u16 {
        self.0
    }
}

impl serde::Serialize for Temperature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_celsius().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Temperature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let celsius = f64::deserialize(deserializer)?;
        Temperature::from_celsius(celsius).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_celsius_zero_is_valid() {
        assert_eq!(Temperature::from_celsius(0.0).unwrap().to_centi_degrees(), 0);
    }

    #[test]
    fn from_celsius_max_is_valid() {
        assert_eq!(Temperature::from_celsius(655.35).unwrap().to_centi_degrees(), 65535);
    }

    #[test]
    fn from_celsius_negative_is_rejected() {
        assert!(Temperature::from_celsius(-0.1).is_err());
    }

    #[test]
    fn from_celsius_above_max_is_rejected() {
        assert!(Temperature::from_celsius(655.36).is_err());
    }

    #[test]
    fn from_celsius_rounds_to_centi_degrees() {
        assert_eq!(Temperature::from_celsius(20.91).unwrap().to_centi_degrees(), 2091);
    }

    #[test]
    fn to_celsius_converts_back() {
        let t = Temperature::from_celsius(34.5).unwrap();
        assert!((t.to_celsius() - 34.5).abs() < 0.01);
    }

    #[test]
    fn to_centi_degrees_is_identity() {
        let t = Temperature::from_centi_degrees(2000);
        assert_eq!(t.to_centi_degrees(), 2000);
    }

    #[test]
    fn from_centi_degrees_preserves_value() {
        let t = Temperature::from_centi_degrees(3001);
        assert_eq!(t.to_centi_degrees(), 3001);
    }

    #[test]
    fn ordering_uses_centi_degrees() {
        let low = Temperature::from_centi_degrees(3000);
        let high = Temperature::from_centi_degrees(3001);
        assert!(low < high);
    }

    #[test]
    fn serialize_outputs_celsius() {
        let t = Temperature::from_celsius(20.5).unwrap();
        assert_eq!(serde_json::to_string(&t).unwrap(), "20.5");
    }

    #[test]
    fn deserialize_parses_celsius() {
        let t: Temperature = serde_json::from_str("34.0").unwrap();
        assert_eq!(t.to_centi_degrees(), 3400);
    }

    #[test]
    fn deserialize_above_max_fails() {
        let result: Result<Temperature, _> = serde_json::from_str("656.0");
        assert!(result.is_err());
    }

    #[test]
    fn roundtrip_serde() {
        let original = Temperature::from_celsius(25.75).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Temperature = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }
}
