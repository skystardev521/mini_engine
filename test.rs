struct St{
    id: u32,
    name: String
}
#[test]
pub test_fn(){
    println("xxxx","xxx");
    let st = St{id:1,name:"name".into()};
    println("st.id:{} st.name:{}", st.id, st.name);
}
