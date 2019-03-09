// SPDX-License-Identifier: AGPL-3.0-only

use crate::db::{IndexedRow, Row, SaveData};
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
    authors: Field,
    barcode: Field,
    discogs_release: Field,
    isbn13: Field,
    lccn: Field,
    oclc_number: Field,
    openlibrary_id: Field,
}

impl ItemSchema {
    fn new() -> ItemSchema {
        use tantivy::schema::{SchemaBuilder, FAST, INT_INDEXED, INT_STORED, STRING, TEXT};

        let mut schema_builder = SchemaBuilder::default();
        let id = schema_builder.add_u64_field("id", INT_INDEXED | INT_STORED | FAST);
        let title = schema_builder.add_text_field("title", TEXT);
        let authors = schema_builder.add_text_field("author", TEXT);
        let barcode = schema_builder.add_text_field("barcode", STRING);
        let discogs_release = schema_builder.add_text_field("discogs", STRING);
        let isbn13 = schema_builder.add_text_field("isbn", STRING);
        let lccn = schema_builder.add_text_field("lccn", STRING);
        let oclc_number = schema_builder.add_text_field("oclc", STRING);
        let openlibrary_id = schema_builder.add_text_field("openlibrary", STRING);
        ItemSchema {
            schema: schema_builder.build(),
            id,
            title,
            authors,
            barcode,
            discogs_release,
            isbn13,
            lccn,
            oclc_number,
            openlibrary_id,
        }
    }
}

lazy_static! {
    static ref SCHEMA: ItemSchema = ItemSchema::new();
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) notes: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) discogs_release: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) isbn13: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) lccn: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) oclc_number: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) openlibrary_id: Option<String>,
}

impl Item {
    pub(crate) fn new(
        classification: String,
        author_sort: &str,
        title: &str,
        language: String,
    ) -> Item {
        Item {
            id: None,

            classification,
            author_sort: author_sort.to_owned(),
            year: None,
            title: title.to_owned(),
            language,

            authors: Vec::new(),
            barcode: None,
            notes: None,

            discogs_release: None,
            isbn13: None,
            lccn: None,
            oclc_number: None,
            openlibrary_id: None,
        }
    }

    fn document(&self) -> Document {
        let mut document = Document::new();

        if let Some(id) = self.id {
            document.add_u64(SCHEMA.id, id);
        }
        document.add_text(SCHEMA.title, &self.title);
        if self.authors.is_empty() {
            document.add_text(SCHEMA.authors, &self.author_sort);
        } else {
            for author in &self.authors {
                document.add_text(SCHEMA.authors, author);
            }
        }

        macro_rules! add_option {
            ($i:ident) => {
                if let Some($i) = &self.$i {
                    document.add_text(SCHEMA.$i, $i)
                }
            }
        }
        add_option!(barcode);
        add_option!(discogs_release);
        add_option!(isbn13);
        add_option!(lccn);
        add_option!(oclc_number);
        add_option!(openlibrary_id);

        document
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
        Ok(SaveData::indexed(
            id,
            serde_cbor::to_vec(self)?,
            Item::id_field(),
            self.document(),
        ))
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
            SCHEMA.authors,
            SCHEMA.barcode,
            SCHEMA.discogs_release,
            SCHEMA.isbn13,
            SCHEMA.lccn,
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
            .then(self.normalize_author().cmp(other.normalize_author()))
            .then(self.year.cmp(&other.year))
            .then(self.title.cmp(&other.title))
            .then(self.language.cmp(&other.language))
            .then(self.id.cmp(&other.id))
    }
}
