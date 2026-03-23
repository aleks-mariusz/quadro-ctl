#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Millicelsius(pub u16);

impl Millicelsius {
    pub fn from_celsius(deg: u16) -> Self {
        Self(deg * 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twenty_five_celsius_converts_to_twenty_five_thousand_millicelsius() {
        assert_eq!(Millicelsius::from_celsius(25), Millicelsius(25000));
    }

    #[test]
    fn zero_celsius_converts_to_zero_millicelsius() {
        assert_eq!(Millicelsius::from_celsius(0), Millicelsius(0));
    }
}
