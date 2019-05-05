use crate::db::{IndexedRow, Row, SaveData};
use failure::{ensure, Fallible};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tantivy::schema::{Document, Field, Schema};

struct UserSchema {
    schema: Schema,
    barcode: Field,
    name: Field,
}

impl UserSchema {
    fn new() -> UserSchema {
        use tantivy::schema::{SchemaBuilder, FAST, INDEXED, STORED, TEXT};

        let mut schema_builder = SchemaBuilder::default();
        let barcode = schema_builder.add_u64_field("barcode", INDEXED | STORED | FAST);
        let name = schema_builder.add_text_field("name", TEXT);
        UserSchema {
            schema: schema_builder.build(),
            barcode,
            name,
        }
    }
}

lazy_static! {
    static ref SCHEMA: UserSchema = UserSchema::new();
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct User {
    pub(crate) barcode: u64,
    pub(crate) name: String,
}

impl User {
    fn document(&self) -> Document {
        let mut document = Document::new();
        document.add_u64(SCHEMA.barcode, self.barcode);
        document.add_text(SCHEMA.name, &self.name);
        document
    }
}

impl Row for User {
    const TREE: &'static str = "users";

    fn load(id: u64, blob: &[u8]) -> Fallible<User> {
        let user: User = serde_cbor::from_slice(blob)?;
        ensure!(id == (user.barcode), "id and barcode do not match");
        Ok(user)
    }

    fn save<F>(&mut self, _id_gen: F) -> Fallible<SaveData>
    where
        F: FnOnce(Option<u64>) -> Fallible<u64>,
    {
        Ok(SaveData::new(self.barcode, serde_cbor::to_vec(self)?)
            .index(User::id_field(), self.document()))
    }
}

impl IndexedRow for User {
    fn schema() -> Schema {
        SCHEMA.schema.clone()
    }

    fn id_field() -> Field {
        SCHEMA.barcode
    }

    fn query_parser_fields() -> Vec<Field> {
        vec![SCHEMA.name]
    }
}
