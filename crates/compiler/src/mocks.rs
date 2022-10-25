use anyhow::Error;
use nova_natives::{block::BlockInfoResolver, table::TableResolver};
use nova_types::table::TableHandle;

/// A dummy storage containing no modules or resources.
#[derive(Debug, Clone)]
pub struct BlankTableViewImpl;

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
        _order: nova_types::iterator::Order,
    ) -> anyhow::Result<u32> {
        Ok(0)
    }

    fn next_key(&mut self, _iterator_id: u32) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(None)
    }
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
