extern crate prost_build;
use std::env;

fn main() {
    let mut out_dir = env::current_dir().unwrap();
    out_dir.push("src");

    let mut vec_proto_file = Vec::new();
    for entry in out_dir.read_dir().unwrap() {
        if let Ok(entry) = entry {
            if entry.path().to_string_lossy().ends_with(".proto") {
                vec_proto_file.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }

    let include_dir = String::from("src");

    let mut prost_build = prost_build::Config::new();
    prost_build.out_dir(out_dir);
    prost_build
        .compile_protos(
            &vec_proto_file[0..], /*&["items.proto"]*/
            &[include_dir],
        )
        .unwrap();
}
