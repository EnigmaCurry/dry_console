use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub sections: HashMap<ConfigSection, ConfigData>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum ConfigSection {
    #[serde(rename = "d.rymcg.tech")]
    DRymcgTech,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ConfigData {
    DRymcgTech(DRymcgTechConfig),
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct DRymcgTechConfig {
    pub installed: bool,
    pub root_path: Option<String>,
}
impl DRymcgTechConfig {
    pub fn validate(&self) -> Result<bool, String> {
        if self.installed && self.root_path.is_none() {
            return Err("installed cannot be true if root_path is None".into());
        }
        Ok(true)
    }
}
