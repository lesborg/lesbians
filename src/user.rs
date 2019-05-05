// SPDX-License-Identifier: AGPL-3.0-only

use crate::db::{IndexedRow, Row, SaveData};
use failure::{ensure, Fallible};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sled::IVec;
use std::collections::HashMap;
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

fn return_false() -> bool {
    false
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn bool_is_false(b: &bool) -> bool {
    !b
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct User {
    pub(crate) barcode: u64,
    pub(crate) name: String,
    #[serde(default = "return_false")]
    #[serde(skip_serializing_if = "bool_is_false")]
    pub(crate) admin: bool,
}

impl User {
    fn document(&self) -> Document {
        let mut document = Document::new();
        document.add_u64(SCHEMA.barcode, self.barcode);
        document.add_text(SCHEMA.name, &self.name);
        document
    }

    #[cfg(test)]
    pub(crate) fn test_user() -> User {
        User {
            barcode: 0,
            name: "test user".to_owned(),
            admin: false,
        }
    }
}

impl Row for User {
    const TREE: &'static str = "users";

    fn load(id: u64, blob: &[u8], _secondary: HashMap<&'static str, IVec>) -> Fallible<User> {
        let user: User = serde_cbor::from_slice(blob)?;
        ensure!(id == user.barcode, "id and barcode do not match");
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

#[cfg(test)]
mod tests {
    use crate::db::Db;
    use crate::user::User;
    use failure::Fallible;

    #[test]
    fn test() -> Fallible<()> {
        let mut db = Db::open_memory()?;
        let mut user = User::test_user();
        db.save(&mut user)?;

        let loaded_user: User = db.load(user.barcode)?.unwrap();
        assert_eq!(user, loaded_user);

        let query_result: Vec<User> = db.query("test")?;
        assert_eq!(query_result.len(), 1);
        assert_eq!(user, query_result[0]);

        Ok(())
    }
}
