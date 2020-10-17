use std::env;

fn main() {
    let library_name = "hiredis";

    let mut cur_dir = env::current_dir().unwrap();

    cur_dir.push("hiredis");

    let cur_src_dir = cur_dir.as_path();

    println!("cur_src_path={}", cur_src_dir.to_string_lossy());

    println!("cargo:rustc-link-lib=static={}", library_name);
    println!(
        "cargo:rustc-link-search=native={}",
        env::join_paths(&[cur_src_dir]).unwrap().to_str().unwrap()
    );
    //println 输出 在 target/debug/build/hiredis-sys-e9d668c60b255fa4/output  文件名
}
