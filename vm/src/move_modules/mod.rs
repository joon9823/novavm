use std::io::Write;
use tempfile::{NamedTempFile, TempPath};

pub fn move_stdlib_files() -> Vec<TempPath> {
    let files: Vec<&str> = vec![
        include_str!("move_stdlib/sources/ascii.move"),
        include_str!("move_stdlib/sources/bcs.move"),
        include_str!("move_stdlib/sources/bit_vector.move"),
        include_str!("move_stdlib/sources/error.move"),
        include_str!("move_stdlib/sources/fixed_point32.move"),
        include_str!("move_stdlib/sources/hash.move"),
        include_str!("move_stdlib/sources/option.move"),
        include_str!("move_stdlib/sources/signer.move"),
        include_str!("move_stdlib/sources/string.move"),
        include_str!("move_stdlib/sources/unit_test.move"),
        include_str!("move_stdlib/sources/vector.move"),
    ];

    files
        .iter()
        .map(|contents| {
            let mut file = NamedTempFile::new().unwrap();
            write!(file, "{}", contents).unwrap();

            file.into_temp_path()
        })
        .collect()
}

pub fn move_nursery_files() -> Vec<TempPath> {
    let files: Vec<&str> = vec![
        include_str!("move_nursery/sources/acl.move"),
        include_str!("move_nursery/sources/capability.move"),
        include_str!("move_nursery/sources/compare.move"),
        include_str!("move_nursery/sources/debug.move"),
        include_str!("move_nursery/sources/errors.move"),
        include_str!("move_nursery/sources/event.move"),
        include_str!("move_nursery/sources/guid.move"),
        include_str!("move_nursery/sources/offer.move"),
        include_str!("move_nursery/sources/role.move"),
        include_str!("move_nursery/sources/vault.move"),
    ];

    files
        .iter()
        .map(|contents| {
            let mut file = NamedTempFile::new().unwrap();
            write!(file, "{}", contents).unwrap();

            file.into_temp_path()
        })
        .collect()
}

pub fn nova_stdlib_files() -> Vec<TempPath> {
    let files: Vec<&str> = vec![
        include_str!("nova_stdlib/sources/account.move"),
        include_str!("nova_stdlib/sources/block.move"),
        include_str!("nova_stdlib/sources/code.move"),
        include_str!("nova_stdlib/sources/coin.move"),
        include_str!("nova_stdlib/sources/comparator.move"),
        include_str!("nova_stdlib/sources/simple_map.move"),
        include_str!("nova_stdlib/sources/table_with_length.move"),
        include_str!("nova_stdlib/sources/table.move"),
        include_str!("nova_stdlib/sources/type_info.move"),
        include_str!("nova_stdlib/sources/util.move"),
    ];

    files
        .iter()
        .map(|contents| {
            let mut file = NamedTempFile::new().unwrap();
            write!(file, "{}", contents).unwrap();

            file.into_temp_path()
        })
        .collect()
}
