//use socket::utils;
//use utils::bytes;



fn main() {
    //utils::setsockopt()
    println!("run");

    let mut hm: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
    hm.insert("key", 999);

    /*
    tcp.Read(|data|{
        match hm.get(&"key") {
            Some(&val)=>println!("t1:{:?} ,val:{}", data, v),
            None=>println!("t1:{:?} ,val: None", data),
        }
    })
    */
}
