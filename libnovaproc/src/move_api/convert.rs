use crate::move_api::move_types::MoveResource;

use anyhow::Result;
use move_deps::move_core_types::{language_storage::StructTag, resolver::MoveResolver};
use move_deps::move_resource_viewer::MoveValueAnnotator;
use nova_natives::table::TableResolver;

use std::str::FromStr;

/// The Move converter for converting Move types to JSON
///
/// This reads the underlying BCS types and ABIs to convert them into
/// JSON outputs
pub struct MoveConverter<'a, R: ?Sized> {
    inner: MoveValueAnnotator<'a, R>,
}

#[allow(dead_code)]
impl<'a, R: MoveResolver + TableResolver + ?Sized> MoveConverter<'a, R> {
    pub fn new(inner: &'a R) -> Self {
        Self {
            inner: MoveValueAnnotator::new(inner),
        }
    }

    pub fn try_into_resource<'b>(&self, struct_tag: &str, bytes: &'b [u8]) -> Result<MoveResource> {
        let struct_tag = StructTag::from_str(struct_tag)?;
        self.inner.view_resource(&struct_tag, bytes)?.try_into()
    }
}
