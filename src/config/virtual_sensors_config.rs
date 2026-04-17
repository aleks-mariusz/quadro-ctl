use serde::Deserialize;
use std::collections::HashMap;

use crate::protocol::Temperature;

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct VirtualSensorsConfig {
    pub sensors: HashMap<String, Temperature>,
}

impl VirtualSensorsConfig {
    pub fn by_index(&self) -> Result<Vec<(usize, Temperature)>, String> {
        let mut result = Vec::new();
        for (key, temp) in &self.sensors {
            let index = parse_virtual_label(key)?;
            result.push((index, *temp));
        }
        Ok(result)
    }
}

fn parse_virtual_label(label: &str) -> Result<usize, String> {
    let num = label
        .strip_prefix("virtual")
        .ok_or_else(|| format!("unknown virtual sensor label: {}", label))?;
    let n: usize = num
        .parse()
        .map_err(|_| format!("invalid virtual sensor label: {}", label))?;
    if n < 1 || n > 16 {
        return Err(format!("virtual sensor index {} out of range 1-16", n));
    }
    Ok(n - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_flat_json() {
        let json = r#"{"virtual1": 30.0, "virtual4": 45.5}"#;
        let config: VirtualSensorsConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.sensors.len(), 2);
    }

    #[test]
    fn parse_virtual_label_valid() {
        assert_eq!(parse_virtual_label("virtual1").unwrap(), 0);
        assert_eq!(parse_virtual_label("virtual16").unwrap(), 15);
    }

    #[test]
    fn parse_virtual_label_out_of_range() {
        assert!(parse_virtual_label("virtual0").is_err());
        assert!(parse_virtual_label("virtual17").is_err());
    }

    #[test]
    fn parse_virtual_label_malformed() {
        assert!(parse_virtual_label("sensor1").is_err());
        assert!(parse_virtual_label("virtualX").is_err());
    }
}
