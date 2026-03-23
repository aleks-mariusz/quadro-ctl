use crate::error::QuadroError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Percentage(u8);

impl Percentage {
    pub fn new(value: u8) -> Result<Self, QuadroError> {
        if value > 100 {
            return Err(QuadroError::ValueOutOfRange {
                field: "percentage",
                value,
                max: 100,
            });
        }
        Ok(Self(value))
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

impl<'de> serde::Deserialize<'de> for Percentage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Percentage::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_valid() {
        assert_eq!(Percentage::new(0).unwrap().value(), 0);
    }

    #[test]
    fn hundred_is_valid() {
        assert_eq!(Percentage::new(100).unwrap().value(), 100);
    }

    #[test]
    fn fifty_is_valid() {
        assert_eq!(Percentage::new(50).unwrap().value(), 50);
    }

    #[test]
    fn above_hundred_is_rejected() {
        assert!(Percentage::new(101).is_err());
    }

    #[test]
    fn max_u8_is_rejected() {
        assert!(Percentage::new(255).is_err());
    }

    #[test]
    fn deserialize_valid_value() {
        let p: Percentage = serde_json::from_str("75").unwrap();
        assert_eq!(p.value(), 75);
    }

    #[test]
    fn deserialize_above_hundred_fails() {
        let result: Result<Percentage, _> = serde_json::from_str("101");
        assert!(result.is_err());
    }
}
