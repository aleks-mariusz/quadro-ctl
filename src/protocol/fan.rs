use super::constants::FAN_CTRL_OFFSETS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanId {
    Fan1,
    Fan2,
    Fan3,
    Fan4,
}

impl FanId {
    pub fn offset(&self) -> usize {
        FAN_CTRL_OFFSETS[self.index()]
    }

    pub fn index(&self) -> usize {
        match self {
            FanId::Fan1 => 0,
            FanId::Fan2 => 1,
            FanId::Fan3 => 2,
            FanId::Fan4 => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanMode {
    Manual,
    Curve,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fan1_offset_is_0x36() {
        assert_eq!(FanId::Fan1.offset(), 0x36);
    }

    #[test]
    fn fan2_offset_is_0x8b() {
        assert_eq!(FanId::Fan2.offset(), 0x8b);
    }

    #[test]
    fn fan3_offset_is_0xe0() {
        assert_eq!(FanId::Fan3.offset(), 0xe0);
    }

    #[test]
    fn fan4_offset_is_0x135() {
        assert_eq!(FanId::Fan4.offset(), 0x135);
    }
}
