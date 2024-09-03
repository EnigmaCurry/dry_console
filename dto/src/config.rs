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
    #[serde(default)]
    pub installed: bool,
    #[serde(default)]
    pub root_path: Option<String>,
    #[serde(default)]
    pub suggested_root_path: Option<String>,
}

impl DRymcgTechConfig {
    pub fn validate(&self) -> Result<bool, String> {
        if self.installed && self.root_path.is_none() {
            Err("installed cannot be true if root_path is None".into())
        } else if self.root_path.is_some() && !self.installed {
            Err("root_path cannot be Some if installed == false".into())
        } else if self.root_path.is_some() && self.suggested_root_path.is_some() {
            Err("suggested_root_path should not be set if root_path is.".into())
        } else if self.suggested_root_path.is_none() && self.root_path.is_none() {
            Err("suggested_root_path should be Some if root_path is None".into())
        } else {
            Ok(true)
        }
    }
}
