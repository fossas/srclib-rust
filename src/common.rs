extern crate cargo;
extern crate rustc_serialize;
extern crate semver;

use self::cargo::core::{Dependency};
use self::cargo::core::dependency::{Platform, Kind};
use self::cargo::ops::{ExportInfo};
use self::rustc_serialize::{Encodable, Encoder};

macro_rules! println_stderr(
  ($($arg:tt)*) => { {
    let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
    r.expect("failed printing to stderr");
  } }
);

pub const PACKAGE_TYPE: &'static str = "RustCargoPackage";

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
  Scope: Kind,
  DefaultFeatures: bool,
  Features: Vec<String>,
  Platform: Option<Platform>
}

impl ResolvedDependency {
  pub fn new(dependency: &Dependency,
             version: Option<EncodableVersion>) -> ResolvedDependency {
    let platform: Option<Platform>;
    match dependency.platform() {
      Some(p) => platform = Some(p.clone()),
      None => platform = None,
    }
    ResolvedDependency {
      Name: dependency.name().to_string(),
      Version: version,
      Optional: dependency.is_optional(),
      Scope: dependency.kind(),
      DefaultFeatures: dependency.uses_default_features(),
      Features: dependency.features().iter().map(|dep| dep.clone()).collect(),
      Platform: platform
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
  Data: Option<ExportInfo>
}

impl SourceUnit {
  pub fn new(name: String,
             kind: String,
             repo: Option<String>,
             files: Vec<String>,
             dir: String,
             dependencies: Option<Vec<ResolvedDependency>>,
             data: Option<ExportInfo>) -> SourceUnit {
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
