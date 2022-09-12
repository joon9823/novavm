
use crate::vm::{
    message::{Message, Module},
};
use move_deps::move_core_types::{
    account_address::AccountAddress,
};

use std::{fs, os::unix::prelude::OsStrExt};

fn generate_messages(dir : &str)->Vec<Message>{
    let paths = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect::<Vec<_>>();
    let mut messages : Vec<Message>  = vec![];
    for path in paths{
        println!("path: {:?}", path);
        let code = path.as_path().as_os_str().as_bytes().to_vec();
        let message = Message::new_module(
            AccountAddress::ONE,
            Module::new(code),
        );
        messages.push(message);
    }
    messages
}

pub fn publish_move_stdlib()->Vec<Message>{
    let dir = "./src/framework/move-stdlib/build/MoveStdlib/bytecode_modules";
    generate_messages(dir)
}
pub fn publish_move_stdlib_nursery()->Vec<Message>{
    let dir = "./src/framework/move-stdlib/nursery/build/MoveNursery/bytecode_modules";
    generate_messages(dir)
}


#[test]
fn test_publish_move_stdlib(){
    let messages = publish_move_stdlib();
    assert_eq!(messages.len(), 10);
}

#[test]
fn test_publish_move_stdlib_nursery(){
    let messages = publish_move_stdlib_nursery();
    assert_eq!(messages.len(), 11);
}