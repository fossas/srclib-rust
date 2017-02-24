extern crate cargo;
extern crate rustc_serialize;
extern crate semver;

use self::cargo::core::{Dependency, SourceId};
use self::cargo::core::dependency::{Platform, Kind};
use self::cargo::ops::{ExportInfo};
use self::cargo::ops::{OutputMetadataOptions};
use self::rustc_serialize::{Encodable, Encoder};

macro_rules! println_stderr(
  ($($arg:tt)*) => { {
    let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
    r.expect("failed printing to stderr");
  } }
);

pub const PACKAGE_TYPE: &'static str = "RustCargoPackage";

pub fn get_output_metadata_options() -> OutputMetadataOptions {
  OutputMetadataOptions {
    features: vec![],
    no_default_features: false,
    all_features: false,
    no_deps: false,
    version: 1,
  }
}

pub struct EncodableVersion {
  version: semver::Version
}

impl Encodable for EncodableVersion {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    format!("{}", self.version).encode(s)
  }
}

impl EncodableVersion {
  pub fn new(version: &semver::Version) -> EncodableVersion {
    EncodableVersion {
      version: version.clone()
    }
  }
}

#[allow(dead_code, non_snake_case)]
#[derive(RustcEncodable)]
pub struct ResolvedDependency {
  Name: String,
  Version: Option<EncodableVersion>,
  Optional: bool,
  Source: SourceId,
  Scope: Option<String>,
  DefaultFeatures: bool,
  Features: Vec<String>,
  Platform: Option<Platform>
}

impl ResolvedDependency {
  pub fn new(dependency: &Dependency,
             version: Option<EncodableVersion>) -> ResolvedDependency {
    ResolvedDependency {
      Name: dependency.name().to_string(),
      Version: version,
      Optional: dependency.is_optional(),
      Source: dependency.source_id().clone(),
      Scope: match dependency.kind() {
            Kind::Normal => Some("normal".to_string()),
            Kind::Development => Some("dev".to_string()),
            Kind::Build => Some("build".to_string()),
        },
      DefaultFeatures: dependency.uses_default_features(),
      Features: dependency.features().iter().map(|dep| dep.clone()).collect(),
      Platform: match dependency.platform() {
        Some(p) => Some(p.clone()),
        None => None,
      }
    }
  }
}

#[allow(dead_code, non_snake_case)]
#[derive(RustcEncodable)]
pub struct SourceUnitMetadata {
  pub ExportInfo: Option<ExportInfo>,
  pub License: Option<String>,
}

impl SourceUnitMetadata {
  pub fn new(exportinfo: Option<ExportInfo>,
             license: Option<String>) -> SourceUnitMetadata {
    SourceUnitMetadata {
      ExportInfo: exportinfo,
      License: license,
    }
  }
}

#[allow(dead_code, non_snake_case)]
#[derive(RustcEncodable)]
pub struct SourceUnit {
  Name: String,
  Type: String,
  Repo: Option<String>,
  Files: Vec<String>,
  Dir: String,
  Dependencies: Option<Vec<ResolvedDependency>>,
  Data: Option<SourceUnitMetadata>
}

impl SourceUnit {
  pub fn new(name: String,
             kind: String,
             repo: Option<String>,
             files: Vec<String>,
             dir: String,
             dependencies: Option<Vec<ResolvedDependency>>,
             data: Option<SourceUnitMetadata>) -> SourceUnit {
    SourceUnit {
      Name: name,
      Type: kind,
      Repo: repo,
      Files: files,
      Dir: dir,
      Dependencies: dependencies,
      Data: data
    }
  }
}
