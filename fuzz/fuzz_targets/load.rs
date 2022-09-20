#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = git_rs::objects::Type::Blob.load(&mut data.clone());
    let _ = git_rs::objects::Type::Commit.load(&mut data.clone());
    let _ = git_rs::objects::Type::Tag.load(&mut data.clone());
    let _ = git_rs::objects::Type::Tree.load(&mut data.clone());
});
