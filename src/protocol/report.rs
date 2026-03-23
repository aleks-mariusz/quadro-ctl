use serde::Serialize;
use std::collections::BTreeMap;

use crate::config::{FanConfig, FanLabel, QuadroConfig};

#[derive(Serialize)]
pub struct Report {
    pub fans: BTreeMap<FanLabel, FanConfig>,
}

impl Report {
    pub fn with_config(&self, config: &QuadroConfig) -> Report {
        let fans = self.fans.iter()
            .map(|(label, fc)| (*label, config.fans.get(label).cloned().unwrap_or_else(|| fc.clone())))
            .collect();
        Report { fans }
    }
}
