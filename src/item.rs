// SPDX-License-Identifier: AGPL-3.0-only

use crate::date::PartialDate;
use crate::db::{IndexedRow, Row, SaveData};
use crate::format::Format;
use crate::isbn::isbn13_to_isbn10;
use crate::lesb::LESBClassification;
use crate::location::Location;
use failure::Fallible;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use tantivy::schema::{Field, Schema};
use tantivy::Document;

struct ItemSchema {
    schema: Schema,
    id: Field,
    title: Field,
    format: Field,
    volume: Field,
    issue: Field,
    location: Field,
    author: Field,
    discogs_release: Field,
    isbn: Field,
    issn: Field,
    lccn: Field,
    mbid: Field,
    oclc_number: Field,
    openlibrary_id: Field,
}

impl ItemSchema {
    #[allow(clippy::similar_names)]
    fn new() -> ItemSchema {
        use tantivy::schema::{SchemaBuilder, FAST, INDEXED, STORED, STRING, TEXT};

        let mut schema_builder = SchemaBuilder::default();
        let id = schema_builder.add_u64_field("id", INDEXED | STORED | FAST);
        let title = schema_builder.add_text_field("title", TEXT);
        let format = schema_builder.add_text_field("format", STRING);
        let volume = schema_builder.add_text_field("volume", STRING);
        let issue = schema_builder.add_text_field("issue", STRING);
        let location = schema_builder.add_text_field("location", STRING);
        let author = schema_builder.add_text_field("author", TEXT);
        let discogs_release = schema_builder.add_text_field("discogs", STRING);
        let isbn = schema_builder.add_text_field("isbn", STRING);
        let issn = schema_builder.add_text_field("issn", STRING);
        let lccn = schema_builder.add_text_field("lccn", STRING);
        let mbid = schema_builder.add_text_field("mbid", STRING);
        let oclc_number = schema_builder.add_text_field("oclc", STRING);
        let openlibrary_id = schema_builder.add_text_field("openlibrary", STRING);
        ItemSchema {
            schema: schema_builder.build(),
            id,
            title,
            format,
            volume,
            issue,
            location,
            author,
            discogs_release,
            isbn,
            issn,
            lccn,
            mbid,
            oclc_number,
            openlibrary_id,
        }
    }
}

lazy_static! {
    static ref SCHEMA: ItemSchema = ItemSchema::new();
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Author {
    name: String,
    sort_name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct JoinPhrase(pub String);

impl JoinPhrase {
    fn is_default(&self) -> bool {
        *self == JoinPhrase::default()
    }
}

impl Default for JoinPhrase {
    fn default() -> JoinPhrase {
        JoinPhrase(", ".to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Credit {
    #[serde(flatten)]
    author: Author,
    #[serde(skip_serializing_if = "Option::is_none")]
    credited_as: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "JoinPhrase::is_default")]
    join_phrase: JoinPhrase,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Item {
    #[serde(skip)]
    id: Option<u64>,

    pub(crate) classification: LESBClassification,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) authors: Vec<Credit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) original_date: Option<PartialDate>,
    pub(crate) title: String,
    pub(crate) language: String,
    pub(crate) format: Format,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) volume_and_issue: Option<(u64, u64)>,
    pub(crate) location: Location,

    /// The inventory control barcode for this item. This is not necessarily the ISBN or UPC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) barcode: Option<String>,
    /// Free-form notes about this item..
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) notes: Option<String>,

    /// Discogs release ID for identifying a release of a musical work.
    ///
    /// This is used to identify a specific released artifact, e.g. a vinyl record vs. a CD for the
    /// same album.
    ///
    /// [Wikidata property P2206](https://www.wikidata.org/wiki/Property:P2206)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) discogs_release: Option<String>,

    /// ISBN-13 for identifying a book. LESBIANS only stores the 13-digit ISBN but has some smarts
    /// around converting 10-digit ISBNs.
    ///
    /// [Wikidata property P212](https://www.wikidata.org/wiki/Property:P212)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) isbn13: Option<String>,

    /// ISSN for identifying a serial.
    ///
    /// [Wikidata property P236](https://www.wikidata.org/wiki/Property:P236)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) issn: Option<String>,

    /// Library of Congress Control Number for identifying a bibliographic record.
    ///
    /// [Wikidata property P1144](https://www.wikidata.org/wiki/Property:P1144)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) lccn: Option<String>,

    /// MusicBrainz release group ID for identifying a musical work.
    ///
    /// [Wikidata property P436](https://www.wikidata.org/wiki/Property:P436)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) musicbrainz_release_group: Option<String>,

    /// OCLC control number for identifying a bibliographic record.
    ///
    /// [Wikidata property P243](https://www.wikidata.org/wiki/Property:P243)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) oclc_number: Option<String>,

    /// Open Library ID for identifying a book.
    ///
    /// [Wikidata property P648](https://www.wikidata.org/wiki/Property:P648)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) openlibrary_id: Option<String>,
}

impl Item {
    fn document(&self) -> Document {
        let mut document = Document::new();

        if let Some(id) = self.id {
            document.add_u64(SCHEMA.id, id);
        }
        document.add_text(SCHEMA.title, &self.title);
        for term in self.format.search_terms() {
            document.add_text(SCHEMA.format, term);
        }
        document.add_text(
            SCHEMA.location,
            &serde_plain::to_string(&self.location).unwrap(),
        );
        for credit in &self.authors {
            document.add_text(SCHEMA.author, &credit.author.name);
            if let Some(credited_as) = &credit.credited_as {
                document.add_text(SCHEMA.author, &credited_as);
            }
        }

        if let Some((volume, issue)) = self.volume_and_issue {
            document.add_text(SCHEMA.volume, &volume.to_string());
            document.add_text(SCHEMA.issue, &issue.to_string());
        }

        macro_rules! add_option {
            ($i:ident) => {
                if let Some($i) = &self.$i {
                    document.add_text(SCHEMA.$i, $i)
                }
            };
        }
        add_option!(discogs_release);
        add_option!(issn);
        add_option!(lccn);
        add_option!(oclc_number);
        add_option!(openlibrary_id);

        if let Some(isbn13) = &self.isbn13 {
            document.add_text(SCHEMA.isbn, isbn13);
            if let Some(isbn10) = isbn13_to_isbn10(isbn13) {
                document.add_text(SCHEMA.isbn, &isbn10);
            }
        }

        if let Some(mbid) = &self.musicbrainz_release_group {
            document.add_text(SCHEMA.mbid, &mbid);
        }

        document
    }

    fn author_sort(&self) -> String {
        let mut s = String::new();
        for credit in &self.authors {
            s.push_str(&credit.author.sort_name);
            s.push_str(&credit.join_phrase.0);
        }
        s
    }

    pub(crate) fn call_number(&self) -> String {
        // Part 1: LESB classification
        let mut call_number = self.classification.to_string();
        call_number.push_str(" ");

        // Part 2: First 5 characters of author (or title, if no authors)
        if self.authors.is_empty() {
            call_number.push_str(&self.title);
        } else {
            let author_normalized: String = self
                .author_sort()
                .chars()
                .filter_map(|c| {
                    if c.is_alphanumeric() {
                        deunicode::deunicode_char(c).map(str::chars)
                    } else {
                        None
                    }
                })
                .flatten()
                .map(|c| c.to_ascii_uppercase())
                .take(5)
                .collect();
            call_number.push_str(&author_normalized);
        }
        call_number.push_str(" ");

        // Part 3: Original year of publication (or _ if data missing)
        match self.original_date {
            Some(date) => call_number.push_str(&date.year().to_string()),
            None => call_number.push_str("_"),
        }
        call_number.push_str(" ");

        // Part 4: Language
        call_number.push_str(&self.language);

        call_number
    }
}

impl Row for Item {
    const TREE: &'static str = "items";

    fn load(id: u64, blob: &[u8]) -> Fallible<Item> {
        let mut item: Item = serde_cbor::from_slice(blob)?;
        item.id = Some(id);
        Ok(item)
    }

    fn save<F>(&mut self, id_gen: F) -> Fallible<SaveData>
    where
        F: FnOnce(Option<u64>) -> Fallible<u64>,
    {
        let id = id_gen(self.id)?;
        self.id = Some(id);
        let mut save_data =
            SaveData::new(id, serde_cbor::to_vec(self)?).index(Item::id_field(), self.document());
        if let Some(barcode) = &self.barcode {
            save_data = save_data.reverse_lookup("item_barcode", barcode.as_bytes().to_vec());
        }
        Ok(save_data)
    }
}

impl IndexedRow for Item {
    fn schema() -> Schema {
        SCHEMA.schema.clone()
    }

    fn id_field() -> Field {
        SCHEMA.id
    }

    fn query_parser_fields() -> Vec<Field> {
        vec![
            SCHEMA.title,
            SCHEMA.format,
            SCHEMA.volume,
            SCHEMA.issue,
            SCHEMA.author,
            SCHEMA.discogs_release,
            SCHEMA.isbn,
            SCHEMA.issn,
            SCHEMA.lccn,
            SCHEMA.mbid,
            SCHEMA.oclc_number,
            SCHEMA.openlibrary_id,
        ]
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
            .then(if self.authors.is_empty() {
                self.title.cmp(&other.title)
            } else {
                self.author_sort().cmp(&other.author_sort())
            })
            .then(self.original_date.cmp(&other.original_date))
            .then(self.volume_and_issue.cmp(&other.volume_and_issue))
            .then(if self.authors.is_empty() {
                ().cmp(&())
            } else {
                self.title.cmp(&other.title)
            })
            .then(self.language.cmp(&other.language))
            .then(self.id.cmp(&other.id))
    }
}
