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
    /// The active workstation directory containing the d.rymcg.tech git repo.
    pub root_dir: Option<String>,
    /// The previous uninstalled directory, which may or may not exist anymore.
    pub previous_root_dir: Option<String>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct DRymcgTechConfigState {
    pub config: DRymcgTechConfig,
    #[serde(default)]
    pub installed: bool,
    #[serde(default)]
    pub suggested_root_dir: Option<String>,
    #[serde(default)]
    pub candidate_root_dir: Option<String>,
}
