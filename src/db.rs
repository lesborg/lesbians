// SPDX-License-Identifier: AGPL-3.0-only

use crate::item::Item;
use failure::{ensure, Fallible};
use serde::{Deserialize, Serialize};
use sled::Tree;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema};
use tantivy::{DocAddress, Document, Index, IndexWriter, Score, Term};

fn id_to_bytes(id: u64) -> [u8; 8] {
    id.to_ne_bytes()
}

fn id_to_u64(id: &[u8]) -> Fallible<u64> {
    ensure!(id.len() == 8, "row ID {:?} is incorrect length", id);
    let mut array = [0; 8];
    array.copy_from_slice(id);
    Ok(u64::from_ne_bytes(array))
}

fn open_or_create_index<T: IndexedRow>(path: &Path) -> Fallible<(Index, IndexWriter)> {
    let path = path.join("idx").join(T::TREE);
    fs::create_dir_all(&path)?;
    let index = Index::open_or_create(MmapDirectory::open(&path)?, T::schema())?;
    let index_writer = index.writer(50_000_000)?;
    Ok((index, index_writer))
}

#[derive(Debug)]
pub(crate) struct SaveData {
    pub(crate) id: u64,
    pub(crate) blob: Vec<u8>,
    pub(crate) index: Option<IndexData>,
}

#[derive(Debug)]
pub(crate) struct IndexData {
    pub(crate) id_field: Field,
    pub(crate) document: Document,
}

impl SaveData {
    pub(crate) fn new(id: u64, blob: Vec<u8>) -> SaveData {
        SaveData {
            id,
            blob,
            index: None,
        }
    }

    pub(crate) fn indexed(id: u64, blob: Vec<u8>, id_field: Field, document: Document) -> SaveData {
        SaveData {
            id,
            blob,
            index: Some(IndexData { id_field, document }),
        }
    }
}

pub(crate) trait Row: Sized {
    const TREE: &'static str;

    fn load(id: u64, blob: &[u8]) -> Fallible<Self>;
    fn save<F>(&mut self, id_gen: F) -> Fallible<SaveData>
    where
        F: FnOnce(Option<u64>) -> Fallible<u64>;
}

pub(crate) trait IndexedRow: Row {
    fn schema() -> Schema;
    fn id_field() -> Field;
    fn query_parser_fields() -> Vec<Field>;
}

pub(crate) struct Db {
    sled: sled::Db,
    indices: HashMap<TypeId, (Index, IndexWriter)>,
}

impl Db {
    pub(crate) fn open<P: AsRef<Path>>(path: P) -> Fallible<Db> {
        let mut indices = HashMap::new();
        indices.insert(
            TypeId::of::<Item>(),
            open_or_create_index::<Item>(path.as_ref())?,
        );

        Ok(Db {
            sled: sled::Db::start_default(path.as_ref().join("sled"))?,
            indices,
        })
    }

    fn open_tree<T: Row>(&self) -> Fallible<Arc<Tree>> {
        Ok(self.sled.open_tree(T::TREE.as_bytes().to_vec())?)
    }

    pub(crate) fn load<T: Row>(&self, id: u64) -> Fallible<Option<T>> {
        let tree = self.open_tree::<T>()?;
        Ok(match tree.get(id_to_bytes(id))? {
            Some(value) => Some(T::load(id, &value)?),
            None => None,
        })
    }

    pub(crate) fn save<T: Row>(&mut self, row: &mut T) -> Fallible<()>
    where
        T: 'static,
    {
        let tree = self.open_tree::<T>()?;
        let save_data = row.save(|id_opt| match id_opt {
            Some(id) => Ok(id),
            None => self.sled.generate_id().map_err(failure::Error::from),
        })?;
        tree.set(id_to_bytes(save_data.id), save_data.blob)?;
        if let Some(IndexData { id_field, document }) = save_data.index {
            if let Some((_, ref mut index_writer)) = self.indices.get_mut(&TypeId::of::<T>()) {
                index_writer.prepare_commit()?;
                index_writer.delete_term(Term::from_field_u64(id_field, save_data.id));
                index_writer.add_document(document);
                index_writer.commit()?;
            }
        }
        Ok(())
    }

    pub(crate) fn query<T: IndexedRow>(&mut self, query: &str) -> Fallible<Vec<T>>
    where
        T: 'static,
    {
        let (index, _) = self
            .indices
            .get(&TypeId::of::<T>())
            .ok_or_else(|| failure::err_msg("no index for row type"))?;
        let searcher = index.searcher();

        let query_parser = QueryParser::for_index(&index, T::query_parser_fields());
        let query = query_parser
            .parse_query(query)
            .map_err(tantivy::Error::from)?;

        let top_docs: Vec<(Score, DocAddress)> =
            searcher.search(&query, &TopDocs::with_limit(10))?;
        let mut docs = Vec::with_capacity(top_docs.len());
        for (_, address) in top_docs {
            let doc = searcher.doc(address)?;
            let id = doc
                .get_first(T::id_field())
                .ok_or_else(|| failure::err_msg("document missing id field"))?
                .u64_value();
            docs.push(
                self.load::<T>(id)?
                    .ok_or_else(|| failure::err_msg(format!("failed to find row {}", id)))?,
            );
        }

        Ok(docs)
    }

    pub(crate) fn iter<T: Row>(&self) -> Fallible<Iter<T>> {
        Ok(Iter::new(self.open_tree::<T>()?))
    }

    fn iter_all(&self) -> Fallible<impl Iterator<Item = Fallible<DumpRow>>> {
        Ok(self.iter::<Item>()?.map(|item| item.map(DumpRow::Item)))
    }

    pub(crate) fn dump<W: Write>(&self, writer: W) -> Fallible<()> {
        let mut writer = writer;
        for item in self.iter_all()? {
            serde_json::to_writer(&mut writer, &item?)?;
            writer.write_all(b"\n")?;
        }
        Ok(())
    }

    pub(crate) fn restore<R: Read>(&mut self, reader: R) -> Fallible<()> {
        let stream = serde_json::Deserializer::from_reader(reader).into_iter();
        for row in stream {
            match row? {
                DumpRow::Item(mut item) => self.save(&mut item)?,
            };
        }
        Ok(())
    }
}

impl fmt::Debug for Db {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Db").finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum DumpRow {
    Item(Item),
}

pub(crate) struct Iter<T> {
    tree: Arc<sled::Tree>,
    last_key: Vec<u8>,
    done: bool,
    phantom: PhantomData<T>,
}

impl<T> Iter<T> {
    fn new(tree: Arc<sled::Tree>) -> Iter<T> {
        Iter {
            tree,
            last_key: Vec::new(),
            done: false,
            phantom: PhantomData,
        }
    }
}

impl<T: Row> Iterator for Iter<T> {
    type Item = Fallible<T>;

    fn next(&mut self) -> Option<Fallible<T>> {
        if self.done {
            None
        } else {
            match self.tree.get_gt(&self.last_key) {
                Ok(Some((key, value))) => {
                    let id = id_to_u64(&key);
                    self.last_key = key;
                    Some(id.and_then(|id| T::load(id, &value)))
                }
                Ok(None) => {
                    self.done = true;
                    None
                }
                Err(err) => {
                    self.done = true;
                    Some(Err(err.into()))
                }
            }
        }
    }
}
