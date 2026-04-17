use crate::error::QuadroError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct SensorIndex(u8);

impl SensorIndex {
    pub fn new(value: u8) -> Result<Self, QuadroError> {
        if value > 19 {
            return Err(QuadroError::ValueOutOfRange {
                field: "sensor",
                value,
                max: 19,
            });
        }
        Ok(Self(value))
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

impl<'de> serde::Deserialize<'de> for SensorIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        SensorIndex::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_valid() {
        assert_eq!(SensorIndex::new(0).unwrap().value(), 0);
    }

    #[test]
    fn three_is_valid() {
        assert_eq!(SensorIndex::new(3).unwrap().value(), 3);
    }

    #[test]
    fn four_is_valid() {
        assert_eq!(SensorIndex::new(4).unwrap().value(), 4);
    }

    #[test]
    fn nineteen_is_valid() {
        assert_eq!(SensorIndex::new(19).unwrap().value(), 19);
    }

    #[test]
    fn twenty_is_rejected() {
        assert!(SensorIndex::new(20).is_err());
    }

    #[test]
    fn max_u8_is_rejected() {
        assert!(SensorIndex::new(255).is_err());
    }

    #[test]
    fn deserialize_valid_value() {
        let s: SensorIndex = serde_json::from_str("2").unwrap();
        assert_eq!(s.value(), 2);
    }

    #[test]
    fn deserialize_above_nineteen_fails() {
        let result: Result<SensorIndex, _> = serde_json::from_str("20");
        assert!(result.is_err());
    }
}
