use super::percentage::Percentage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CentiPercent(pub u16);

impl CentiPercent {
    pub fn from_percentage(pct: Percentage) -> Self {
        Self(pct.value() as u16 * 100)
    }

    pub fn to_percentage(self) -> Percentage {
        Percentage::new((self.0 / 100) as u8).unwrap_or_else(|_| {
            Percentage::new(100).expect("100 is always valid")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_percent_converts_to_zero_centi_percent() {
        assert_eq!(CentiPercent::from_percentage(Percentage::new(0).unwrap()), CentiPercent(0));
    }

    #[test]
    fn fifty_percent_converts_to_five_thousand_centi_percent() {
        assert_eq!(CentiPercent::from_percentage(Percentage::new(50).unwrap()), CentiPercent(5000));
    }

    #[test]
    fn hundred_percent_converts_to_ten_thousand_centi_percent() {
        assert_eq!(CentiPercent::from_percentage(Percentage::new(100).unwrap()), CentiPercent(10000));
    }

    #[test]
    fn zero_centi_percent_converts_to_zero_percent() {
        assert_eq!(CentiPercent(0).to_percentage(), Percentage::new(0).unwrap());
    }

    #[test]
    fn five_thousand_centi_percent_converts_to_fifty_percent() {
        assert_eq!(CentiPercent(5000).to_percentage(), Percentage::new(50).unwrap());
    }

    #[test]
    fn ten_thousand_centi_percent_converts_to_hundred_percent() {
        assert_eq!(CentiPercent(10000).to_percentage(), Percentage::new(100).unwrap());
    }
}
