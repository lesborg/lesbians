// SPDX-License-Identifier: AGPL-3.0-only

use crate::db::Row;
use failure::Fallible;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Item {
    #[serde(skip)]
    id: Option<u64>,

    pub(crate) classification: String,
    pub(crate) author_sort: String,
    pub(crate) year: Option<u64>,
    pub(crate) title: String,
    pub(crate) language: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) authors: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) barcode: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) identifiers: Vec<Identifier>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) notes: Option<String>,
}

impl Item {
    pub(crate) fn new(
        classification: String,
        author_sort: &str,
        title: &str,
        language: String,
    ) -> Item {
        Item {
            id: Default::default(),

            classification,
            author_sort: author_sort.to_owned(),
            year: Default::default(),
            title: title.to_owned(),
            language,

            authors: Default::default(),
            barcode: Default::default(),
            identifiers: Default::default(),
            notes: Default::default(),
        }
    }

    fn normalize_author(&self) -> impl Iterator<Item = char> + '_ {
        self.author_sort
            .chars()
            .filter_map(|c| {
                if c.is_alphanumeric() {
                    deunicode::deunicode_char(c)
                } else {
                    None
                }
            })
            .map(str::chars)
            .flatten()
            .map(|c| c.to_ascii_uppercase())
    }

    pub(crate) fn call_number(&self) -> String {
        let author: String = self.normalize_author().take(5).collect();
        if let Some(year) = self.year {
            format!(
                "{} {} {} {}",
                self.classification, author, year, self.language
            )
        } else {
            format!("{} {} {}", self.classification, author, self.language)
        }
    }

    pub(crate) fn author(&self) -> String {
        if self.authors.is_empty() {
            self.author_sort.to_owned()
        } else {
            (&self.authors).join(", ")
        }
    }
}

impl Row for Item {
    const TREE: &'static [u8] = b"items";

    fn load(id: u64, blob: &[u8]) -> Fallible<Item> {
        let mut item: Item = serde_cbor::from_slice(blob)?;
        item.id = Some(id);
        Ok(item)
    }

    fn save<F>(&mut self, id_gen: F) -> Fallible<(u64, Vec<u8>)>
    where
        F: FnOnce(Option<u64>) -> Fallible<u64>,
    {
        let id = id_gen(self.id)?;
        self.id = Some(id);
        Ok((id, serde_cbor::to_vec(self)?))
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        self.classification
            .cmp(&other.classification)
            .then(self.normalize_author().cmp(other.normalize_author()))
            .then(self.year.cmp(&other.year))
            .then(self.title.cmp(&other.title))
            .then(self.language.cmp(&other.language))
            .then(self.id.cmp(&other.id))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum Identifier {
    #[serde(rename = "discogs-master")]
    DiscogsMaster(String),
    #[serde(rename = "discogs-release")]
    DiscogsRelease(String),
    #[serde(rename = "isbn13")]
    ISBN13(String),
    #[serde(rename = "lccn")]
    LCCN(String),
    #[serde(rename = "musicbrainz-release")]
    MusicBrainzRelease(String),
    #[serde(rename = "musicbrainz-release-group")]
    MusicBrainzReleaseGroup(String),
    #[serde(rename = "oclc")]
    OCLC(String),
    #[serde(rename = "openlibrary")]
    OpenLibrary(String),
}
