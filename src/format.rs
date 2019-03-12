// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Format {
    Paperback,
    Hardcover,
    Magazine,
    Zine,
    CD,
    #[serde(rename = "vinyl-12-inch")]
    Vinyl12Inch,
    #[serde(rename = "vinyl-10-inch")]
    Vinyl10Inch,
    #[serde(rename = "vinyl-7-inch")]
    Vinyl7Inch,
    Cassette,
}

impl Format {
    pub(crate) fn search_terms(&self) -> Vec<&'static str> {
        use Format::*;

        match self {
            Paperback => vec!["paperback", "book", "print"],
            Hardcover => vec!["hardcover", "book", "print"],
            Magazine => vec!["magazine", "print"],
            Zine => vec!["zine", "print"],
            CD => vec!["cd", "music"],
            Vinyl12Inch => vec!["vinyl12inch", "vinyl", "music"],
            Vinyl10Inch => vec!["vinyl10inch", "vinyl", "music"],
            Vinyl7Inch => vec!["vinyl7inch", "vinyl", "music"],
            Cassette => vec!["cassette", "music"],
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Format::*;

        write!(
            f,
            "{}",
            match self {
                Paperback => "paperback book",
                Hardcover => "hardcover book",
                Magazine => "magazine",
                Zine => "zine",
                CD => "compact disc",
                Vinyl12Inch => "12-inch vinyl record",
                Vinyl10Inch => "10-inch vinyl record",
                Vinyl7Inch => "7-inch vinyl record",
                Cassette => "audio cassette",
            }
        )
    }
}
