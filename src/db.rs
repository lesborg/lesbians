// SPDX-License-Identifier: AGPL-3.0-only

use crate::item::Item;
use failure::{ensure, Fallible};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::prelude::*;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

fn id_to_bytes(id: u64) -> [u8; 8] {
    id.to_ne_bytes()
}

fn id_to_u64(id: &[u8]) -> Fallible<u64> {
    ensure!(id.len() == 8, "row ID {:?} is incorrect length", id);
    let mut array = [0; 8];
    array.copy_from_slice(id);
    Ok(u64::from_ne_bytes(array))
}

pub(crate) trait Row: Sized {
    const TREE: &'static [u8];

    fn load(id: u64, blob: &[u8]) -> Fallible<Self>;
    fn save<F>(&mut self, id_gen: F) -> Fallible<(u64, Vec<u8>)>
    where
        F: FnOnce(Option<u64>) -> Fallible<u64>;
}

pub(crate) struct Db {
    sled: sled::Db,
}

impl Db {
    pub(crate) fn open<P: AsRef<Path>>(path: P) -> Fallible<Db> {
        Ok(Db {
            sled: sled::Db::start_default(path.as_ref().join("sled"))?,
        })
    }

    pub(crate) fn load<T: Row>(&self, id: u64) -> Fallible<Option<T>> {
        let tree = self.sled.open_tree(T::TREE.to_vec())?;
        Ok(match tree.get(id_to_bytes(id))? {
            Some(value) => Some(T::load(id, &value)?),
            None => None,
        })
    }

    pub(crate) fn save<T: Row>(&self, row: &mut T) -> Fallible<()> {
        let tree = self.sled.open_tree(T::TREE.to_vec())?;
        let (id, blob) = row.save(|id_opt| match id_opt {
            Some(id) => Ok(id),
            None => self.sled.generate_id().map_err(failure::Error::from),
        })?;
        tree.set(id_to_bytes(id), blob)?;
        Ok(())
    }

    pub(crate) fn iter<T: Row>(&self) -> Fallible<Iter<T>> {
        Ok(Iter::new(self.sled.open_tree(T::TREE.to_vec())?))
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

    pub(crate) fn restore<R: BufRead>(&self, reader: R) -> Fallible<()> {
        for line in reader.lines() {
            let row: DumpRow = serde_json::from_str(&line?)?;
            match row {
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
