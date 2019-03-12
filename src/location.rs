// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Location {
    Billy,
    BillyOversize,
    Kitchen,
    VinylShelf,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Location::*;

        write!(
            f,
            "{}",
            match self {
                Billy => "Billy bookcases",
                BillyOversize => "Billy bookcases (oversize)",
                Kitchen => "Kitchen",
                VinylShelf => "Vinyl shelf",
            }
        )
    }
}
