use nova_storage::{state_view::StateView, table_view::TableView};
use std::{
    collections::BTreeMap,
    ops::{Bound, RangeBounds},
};

use move_deps::move_core_types::effects::Op;
use nova_natives::{block::BlockInfoResolver, table::TableResolver};
use nova_types::{
    access_path::AccessPath, iterator::Order, table::TableHandle, write_set::WriteSet,
};

use anyhow::{anyhow, Error};

#[derive(Debug)]
pub struct MockChain {
    map: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl MockChain {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    // not scalable because it simply clones current map
    pub fn create_state(&self) -> MockState {
        MockState {
            map: self.map.clone(),
        }
    }

    pub fn create_api(&self, height: u64, timestamp: u64) -> MockApi {
        MockApi { height, timestamp }
    }

    pub fn commit(&mut self, state: MockState) {
        self.map = state.map;
    }
}

pub struct MockState {
    map: BTreeMap<Vec<u8>, Vec<u8>>,
}

/// The BTreeMap specific key-value pair reference type, as returned by BTreeMap<Vec<u8>, Vec<u8>>::range.
/// This is internal as it can change any time if the map implementation is swapped out.
type BTreeMapRecordRef<'a> = (&'a Vec<u8>, &'a Vec<u8>);

impl MockState {
    fn write_op(&mut self, ref ap: AccessPath, ref blob_opt: Op<Vec<u8>>) {
        match blob_opt {
            Op::New(blob) | Op::Modify(blob) => {
                self.map.insert(ap.to_bytes().unwrap(), blob.clone());
            }
            Op::Delete => {
                self.map.remove(&ap.to_bytes().unwrap());
            }
        }
    }

    pub fn push_write_set(&mut self, write_set: WriteSet) {
        for (ap, blob_opt) in write_set {
            self.write_op(ap, blob_opt)
        }
    }
}

impl StateView for MockState {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(self.map.get(&access_path.to_bytes()?).cloned())
    }
}

pub struct MockTableState<'r> {
    inner: &'r MockState,
    iterators: Vec<Vec<Vec<u8>>>,
}

impl<'r> MockTableState<'r> {
    pub fn new(mock_state: &'r MockState) -> Self {
        Self {
            inner: mock_state,
            iterators: vec![],
        }
    }
}

impl<'r> TableView for MockTableState<'r> {
    fn resolve_table_entry(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let access_path = AccessPath::table_item_access_path(handle.0, key.to_vec()).to_bytes()?;
        Ok(self.inner.map.get(&access_path).cloned())
    }

    fn create_iterator(
        &mut self,
        handle: &TableHandle,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> anyhow::Result<u32> {
        let prefix = AccessPath::table_item_access_path(handle.0, vec![]).to_bytes()?;

        let start = start.map_or(prefix.clone(), |v| {
            let mut prefix = prefix.clone();
            prefix.append(&mut v.to_vec());
            prefix
        });
        let end = end.map_or(prefix_end_bytes(prefix.clone()), |v| {
            let mut prefix = prefix.clone();
            prefix.append(&mut v.to_vec());
            prefix
        });

        let bounds = range_bounds(&start, &end);

        // BTreeMap.range panics if range is start > end.
        // However, this cases represent just empty range and we treat it as such.
        match (bounds.start_bound(), bounds.end_bound()) {
            (Bound::Included(start), Bound::Excluded(end)) if start > end => {
                let iterator_id = self.iterators.len();
                self.iterators.push(vec![]);
                return Ok(iterator_id as u32);
            }
            _ => {}
        }

        let iterator_id = self.iterators.len();

        let prefix_len = prefix.len();
        let iter = self.inner.map.range(bounds);
        self.iterators.push(match order {
            Order::Ascending => iter
                .map(|v| clone_and_format_item(v, prefix_len))
                .collect::<Vec<Vec<u8>>>(),
            Order::Descending => iter
                .rev()
                .map(|v| clone_and_format_item(v, prefix_len))
                .collect(),
        });

        Ok(iterator_id as u32)
    }

    fn next_key(&mut self, iterator_id: u32) -> anyhow::Result<Option<Vec<u8>>> {
        match self.iterators.get_mut(iterator_id as usize) {
            Some(iterator) => Ok(match iterator.get(0).map(|v| v.to_vec()) {
                Some(key_bytes) => {
                    iterator.remove(0);
                    Some(key_bytes)
                }
                None => None,
            }),
            None => Err(anyhow!("iterator not found")),
        }
    }
}

fn prefix_end_bytes(prefix: Vec<u8>) -> Vec<u8> {
    if prefix.len() == 0 {
        return vec![];
    }

    let mut end = prefix;
    loop {
        let last = end.last_mut().unwrap();
        if *last != 255u8 {
            *last += 1u8;
            break;
        }

        end.pop();

        if end.len() == 0 {
            break;
        }
    }

    end
}

fn range_bounds(start: &[u8], end: &[u8]) -> impl RangeBounds<Vec<u8>> {
    (
        Bound::Included(start.to_vec()),
        Bound::Excluded(end.to_vec()),
    )
}

fn clone_and_format_item(item_ref: BTreeMapRecordRef, prefix_length: usize) -> Vec<u8> {
    let (key, _) = item_ref;
    key[prefix_length..].to_vec()
}

pub struct MockApi {
    pub height: u64,
    pub timestamp: u64,
}

impl BlockInfoResolver for MockApi {
    fn get_block_info(&self) -> anyhow::Result<(u64 /* height */, u64 /* timestamp */)> {
        Ok((self.height, self.timestamp))
    }
}

/// A dummy storage containing no modules or resources.
/// only used for unit test
#[derive(Debug, Clone)]
pub struct BlankTableViewImpl;

impl BlankTableViewImpl {
    pub fn new() -> Self {
        Self
    }
}

impl TableResolver for BlankTableViewImpl {
    fn resolve_table_entry(
        &self,
        _handle: &TableHandle,
        _key: &[u8],
    ) -> Result<Option<Vec<u8>>, Error> {
        Ok(None)
    }

    fn create_iterator(
        &mut self,
        _handle: &TableHandle,
        _start: Option<&[u8]>,
        _end: Option<&[u8]>,
        _order: Order,
    ) -> anyhow::Result<u32> {
        Ok(0)
    }

    fn next_key(&mut self, _iterator_id: u32) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(None)
    }
}
