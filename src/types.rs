use std::path::PathBuf;

use derive_builder::Builder;
use serde::Serialize;

#[derive(Debug, Clone, PartialOrd, PartialEq, Serialize)]
pub struct License {
    #[serde(rename = "License")]
    value: String,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Serialize, Builder)]
pub struct SourceUnit {
    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "Type")]
    unit_type: String,

    #[serde(rename = "Repo")]
    repo: Option<String>,

    #[serde(rename = "Files")]
    files: Vec<PathBuf>,

    #[serde(rename = "Dependencies")]
    deps: Option<Vec<ResolvedDependency>>,

    #[serde(rename = "Data")]
    data: Option<License>,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Serialize, Builder)]
pub struct ResolvedDependency {
    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "Version")]
    version: Option<String>,

    #[serde(rename = "Optional")]
    optional: bool,

    #[serde(rename = "Source")]
    source: String,

    #[serde(rename = "Scope")]
    scope: Option<String>,

    #[serde(rename = "DefaultFeatures")]
    default_features: bool,

    #[serde(rename = "Features")]
    features: Vec<String>,

    #[serde(rename = "Path")]
    cargo_toml_path: Option<String>,

    #[serde(rename = "Platform")]
    platform: Option<String>,
}

impl From<String> for License {
    fn from(value: String) -> Self {
        Self { value }
    }
}
