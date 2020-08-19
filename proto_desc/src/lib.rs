use prost::Message;

include!("./snazzy.items.rs");

pub fn create_large_shirt(color: String) -> Shirt {
    let mut shirt = Shirt::default();
    shirt.color = color;
    shirt.set_size(shirt::Size::Large);
    shirt
}

pub fn serialize_shirt(shirt: &Shirt) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(shirt.encoded_len());
    shirt.encode(&mut buf).unwrap();
    buf
}

pub fn deserialize_shirt(buf: &[u8]) -> Result<Shirt, prost::DecodeError> {
    //let mut c = Cursor::new(buf);
    //c.set_position(2);
    Shirt::decode(buf)
}
