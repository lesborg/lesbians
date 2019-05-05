// SPDX-License-Identifier: AGPL-3.0-only

use crate::item::Item;
use crate::user::User;
use failure::{ensure, Fallible};
use serde::{Deserialize, Serialize};
use sled::{IVec, Tree};
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

pub(crate) fn id_to_bytes(id: u64) -> [u8; 8] {
    id.to_ne_bytes()
}

pub(crate) fn id_to_u64(id: &[u8]) -> Fallible<u64> {
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

#[cfg(test)]
fn create_ram_index<T: IndexedRow>() -> Fallible<(Index, IndexWriter)> {
    let index = Index::create_in_ram(T::schema());
    let index_writer = index.writer(50_000_000)?;
    Ok((index, index_writer))
}

#[derive(Debug)]
pub(crate) struct SaveData {
    id: u64,
    blob: Vec<u8>,
    index: Option<IndexData>,
    secondary: HashMap<&'static str, Vec<u8>>,
}

#[derive(Debug)]
pub(crate) struct IndexData {
    id_field: Field,
    document: Document,
}

impl SaveData {
    pub(crate) fn new(id: u64, blob: Vec<u8>) -> SaveData {
        SaveData {
            id,
            blob,
            index: None,
            secondary: HashMap::new(),
        }
    }

    pub(crate) fn index(self, id_field: Field, document: Document) -> SaveData {
        let mut v = self;
        v.index = Some(IndexData { id_field, document });
        v
    }

    pub(crate) fn secondary(self, tree_name: &'static str, blob: Vec<u8>) -> SaveData {
        let mut v = self;
        v.secondary.insert(tree_name, blob);
        v
    }
}

pub(crate) trait Row: Sized {
    const TREE: &'static str;
    const SECONDARY: &'static [&'static str] = &[];

    fn load(id: u64, blob: &[u8], secondary: HashMap<&'static str, IVec>) -> Fallible<Self>;
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
        indices.insert(
            TypeId::of::<User>(),
            open_or_create_index::<User>(path.as_ref())?,
        );

        Ok(Db {
            sled: sled::Db::start_default(path.as_ref().join("sled"))?,
            indices,
        })
    }

    #[cfg(test)]
    pub(crate) fn open_memory() -> Fallible<Db> {
        let mut indices = HashMap::new();
        indices.insert(TypeId::of::<Item>(), create_ram_index::<Item>()?);
        indices.insert(TypeId::of::<User>(), create_ram_index::<User>()?);

        let config = sled::ConfigBuilder::default().temporary(true).build();

        Ok(Db {
            sled: sled::Db::start(config)?,
            indices,
        })
    }

    fn open_tree<T: Row>(&self) -> Fallible<Arc<Tree>> {
        Ok(self.sled.open_tree(T::TREE.as_bytes().to_vec())?)
    }

    fn open_secondary<T: Row>(&self, secondary: &'static str) -> Fallible<Arc<Tree>> {
        Ok(self
            .sled
            .open_tree(format!("{}-{}", T::TREE, secondary).as_bytes().to_vec())?)
    }

    pub(crate) fn load<T: Row>(&self, id: u64) -> Fallible<Option<T>> {
        let tree = self.open_tree::<T>()?;
        Ok(match tree.get(id_to_bytes(id))? {
            Some(value) => {
                let mut map = HashMap::new();
                for tree_name in T::SECONDARY {
                    let tree = self.open_secondary::<T>(tree_name)?;
                    if let Some(v) = tree.get(id_to_bytes(id))? {
                        map.insert(*tree_name, v);
                    }
                }
                Some(T::load(id, &value, map)?)
            }
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
        let id_bytes = id_to_bytes(save_data.id);
        tree.set(id_bytes, save_data.blob)?;
        if let Some(IndexData { id_field, document }) = save_data.index {
            if let Some((_, ref mut index_writer)) = self.indices.get_mut(&TypeId::of::<T>()) {
                index_writer.prepare_commit()?;
                index_writer.delete_term(Term::from_field_u64(id_field, save_data.id));
                index_writer.add_document(document);
                index_writer.commit()?;
            }
        }
        for tree_name in T::SECONDARY {
            let tree = self.open_secondary::<T>(tree_name)?;
            match save_data.secondary.get(tree_name) {
                Some(data) => tree.set(id_bytes, data.as_slice())?,
                None => tree.del(id_bytes)?,
            };
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
        let searcher = index.reader()?.searcher();

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
        let mut map = HashMap::new();
        for tree_name in T::SECONDARY {
            map.insert(*tree_name, self.open_secondary::<T>(tree_name)?);
        }
        Ok(Iter::new(self.open_tree::<T>()?, map))
    }

    fn iter_all(&self) -> Fallible<impl Iterator<Item = Fallible<DumpRow>>> {
        Ok(self
            .iter::<Item>()?
            .map(|item| item.map(DumpRow::from))
            .chain(self.iter::<User>()?.map(|user| user.map(DumpRow::from))))
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
                DumpRow::Item(mut item) => self.save(&mut *item)?,
                DumpRow::User(mut user) => self.save(&mut *user)?,
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
    Item(Box<Item>),
    User(Box<User>),
}

impl From<Item> for DumpRow {
    fn from(x: Item) -> DumpRow {
        DumpRow::Item(Box::new(x))
    }
}

impl From<User> for DumpRow {
    fn from(x: User) -> DumpRow {
        DumpRow::User(Box::new(x))
    }
}

pub(crate) struct Iter<T> {
    tree: Arc<sled::Tree>,
    secondary: HashMap<&'static str, Arc<Tree>>,
    last_key: Vec<u8>,
    done: bool,
    phantom: PhantomData<T>,
}

impl<T> Iter<T> {
    fn new(tree: Arc<Tree>, secondary: HashMap<&'static str, Arc<Tree>>) -> Iter<T> {
        Iter {
            tree,
            secondary,
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
                    let mut secondary = HashMap::new();
                    for (tree_name, tree) in &self.secondary {
                        match tree.get(&key) {
                            Ok(Some(v)) => {
                                secondary.insert(*tree_name, v);
                            }
                            Ok(None) => {}
                            Err(err) => return Some(Err(err.into())),
                        }
                    }
                    self.last_key = key;
                    Some(id.and_then(|id| T::load(id, &value, secondary)))
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
