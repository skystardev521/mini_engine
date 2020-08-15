//extern crate prost_build;
use std::env;

///extern crate prost_build;

fn main() {
    let path = std::path::Path::new("/ShareDir/mini_engine/proto/src");

    let mut files = Vec::new();
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            if entry.path().to_string_lossy().ends_with(".proto") {
                files.push(entry.file_name());
                println!("{:?} {:?}", entry.path(), entry.file_name());
            }
        }
    }
}
