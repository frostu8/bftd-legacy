//! Asset management.

use semver::Version;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Bundle metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Hash, Serialize)]
pub struct Metadata {
    /// The name of the bundle.
    pub name: String,
    /// The version of the bundle.
    pub version: Version,
}

impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.name, self.version)
    }
}
