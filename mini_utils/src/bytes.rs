pub struct Bytes<'a> {
    pos: usize,
    buffer: &'a mut [u8],
}

impl<'a> Bytes<'a> {
    #[inline]
    pub fn new(buffer: &'a mut [u8]) -> Bytes<'a> {
        Bytes {
            pos: 0usize,
            buffer: buffer,
        }
    }

    #[inline]
    pub fn get_pos(&mut self) -> usize {
        self.pos
    }
    #[inline]
    pub fn set_pos(&mut self, pos: usize) {
        let len = self.buffer.len();
        if pos < len {
            self.pos = pos;
        } else {
            self.pos = len - 1;
        }
    }
    #[inline]
    pub fn get_size(&mut self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn write_u8(&mut self, n: u8) {
        self.pos += 1;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        self.buffer[(self.pos - 1)] = n;
    }

    #[inline]
    pub fn write_i8(&mut self, n: i8) {
        self.pos += 1;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        self.buffer[self.pos - 1] = n as u8;
    }

    #[inline]
    pub fn write_u16(&mut self, n: u16) {
        self.pos += 2;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        let ns = n.to_le_bytes();
        self.buffer[self.pos - 2] = ns[0];
        self.buffer[self.pos - 1] = ns[1];
    }
    #[inline]
    pub fn write_i16(&mut self, n: i16) {
        self.pos += 2;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        let ns = n.to_le_bytes();
        self.buffer[self.pos - 2] = ns[0];
        self.buffer[self.pos - 1] = ns[1];
    }

    #[inline]
    pub fn write_u32(&mut self, n: u32) {
        self.pos += 4;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        let ns = n.to_le_bytes();
        self.buffer[self.pos - 4] = ns[0];
        self.buffer[self.pos - 3] = ns[1];
        self.buffer[self.pos - 2] = ns[2];
        self.buffer[self.pos - 1] = ns[3];
    }
    #[inline]
    pub fn write_i32(&mut self, n: i32) {
        self.pos += 4;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        let ns = n.to_le_bytes();
        self.buffer[self.pos - 4] = ns[0];
        self.buffer[self.pos - 3] = ns[1];
        self.buffer[self.pos - 2] = ns[2];
        self.buffer[self.pos - 1] = ns[3];
    }
    #[inline]
    pub fn write_u64(&mut self, n: u64) {
        self.pos += 8;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        let ns = n.to_le_bytes();
        self.buffer[self.pos - 8] = ns[0];
        self.buffer[self.pos - 7] = ns[1];
        self.buffer[self.pos - 6] = ns[2];
        self.buffer[self.pos - 5] = ns[3];
        self.buffer[self.pos - 4] = ns[4];
        self.buffer[self.pos - 3] = ns[5];
        self.buffer[self.pos - 2] = ns[6];
        self.buffer[self.pos - 1] = ns[7];
    }

    #[inline]
    pub fn write_i64(&mut self, n: i64) {
        self.pos += 8;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        let ns = n.to_le_bytes();
        self.buffer[self.pos - 8] = ns[0];
        self.buffer[self.pos - 7] = ns[1];
        self.buffer[self.pos - 6] = ns[2];
        self.buffer[self.pos - 5] = ns[3];
        self.buffer[self.pos - 4] = ns[4];
        self.buffer[self.pos - 3] = ns[5];
        self.buffer[self.pos - 2] = ns[6];
        self.buffer[self.pos - 1] = ns[7];
    }

    #[inline]
    pub fn write_f32(&mut self, n: f32) {
        self.pos += 4;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        let ns = n.to_le_bytes();
        self.buffer[self.pos - 4] = ns[0];
        self.buffer[self.pos - 3] = ns[1];
        self.buffer[self.pos - 2] = ns[2];
        self.buffer[self.pos - 1] = ns[3];
    }

    #[inline]
    pub fn write_f64(&mut self, n: f64) {
        self.pos += 8;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        let ns = n.to_le_bytes();
        self.buffer[self.pos - 8] = ns[0];
        self.buffer[self.pos - 7] = ns[1];
        self.buffer[self.pos - 6] = ns[2];
        self.buffer[self.pos - 5] = ns[3];
        self.buffer[self.pos - 4] = ns[4];
        self.buffer[self.pos - 3] = ns[5];
        self.buffer[self.pos - 2] = ns[6];
        self.buffer[self.pos - 1] = ns[7];
    }

    #[inline]
    pub fn write_bytes(&mut self, n: &[u8]) {
        let len = n.len();
        if len == 0 {
            debug_assert!(false);
            return ();
        }
        self.pos += len;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return ();
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                n.as_ptr(),
                self.buffer[self.pos - len..self.pos].as_mut_ptr(),
                len,
            );
        }
    }

    #[inline]
    pub fn read_u8(&mut self) -> u8 {
        self.pos += 1;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return 0u8;
        }
        self.buffer[self.pos - 1]
    }
    #[inline]
    pub fn read_i8(&mut self) -> i8 {
        self.pos += 1;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return 0i8;
        }
        self.buffer[self.pos - 1] as i8
    }
    #[inline]
    pub fn read_u16(&mut self) -> u16 {
        self.pos += 2;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return 0u16;
        }
        let p: *const u8 = self.buffer[self.pos - 2..].as_ptr();
        #[cfg(target_endian = "little")]
        {
            unsafe { *(p as *const u16) }
        }
        #[cfg(not(target_endian = "little"))]
        {
            unsafe { *(p as *const u16) }.swap_bytes()
        }
    }
    #[inline]
    pub fn read_i16(&mut self) -> i16 {
        self.pos += 2;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return 0i16;
        }
        let p: *const u8 = self.buffer[self.pos - 2..].as_ptr();
        #[cfg(target_endian = "little")]
        {
            unsafe { *(p as *const i16) }
        }
        #[cfg(not(target_endian = "little"))]
        {
            unsafe { *(p as *const i16) }.swap_bytes()
        }
    }
    #[inline]
    pub fn read_u32(&mut self) -> u32 {
        self.pos += 4;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return 0u32;
        }
        let p: *const u8 = self.buffer[self.pos - 4..].as_ptr();
        #[cfg(target_endian = "little")]
        {
            unsafe { *(p as *const u32) }
        }
        #[cfg(not(target_endian = "little"))]
        {
            unsafe { *(p as *const u32) }.swap_bytes()
        }
    }
    #[inline]
    pub fn read_i32(&mut self) -> i32 {
        self.pos += 4;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return 0i32;
        }
        let p: *const u8 = self.buffer[self.pos - 4..].as_ptr();
        #[cfg(target_endian = "little")]
        {
            unsafe { *(p as *const i32) }
        }
        #[cfg(not(target_endian = "little"))]
        {
            unsafe { *(p as *const i32) }.swap_bytes()
        }
    }

    #[inline]
    pub fn read_u64(&mut self) -> u64 {
        self.pos += 8;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return 0u64;
        }
        let p: *const u8 = self.buffer[self.pos - 8..].as_ptr();
        #[cfg(target_endian = "little")]
        {
            unsafe { *(p as *const u64) }
        }
        #[cfg(not(target_endian = "little"))]
        {
            unsafe { *(p as *const u64) }.swap_bytes()
        }
    }
    #[inline]
    pub fn read_i64(&mut self) -> i64 {
        self.pos += 8;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return 0i64;
        }
        let p: *const u8 = self.buffer[self.pos - 8..].as_ptr();
        #[cfg(target_endian = "little")]
        {
            unsafe { *(p as *const i64) }
        }
        #[cfg(not(target_endian = "little"))]
        {
            unsafe { *(p as *const i64) }.swap_bytes()
        }
    }

    #[inline]
    pub fn read_f32(&mut self) -> f32 {
        self.pos += 4;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return 0f32;
        }
        #[cfg(target_endian = "little")]
        {
            let p: *const u8 = self.buffer[self.pos - 4..].as_ptr();
            unsafe { *(p as *const f32) }
        }
        #[cfg(not(target_endian = "little"))]
        {
            let p: *const u8 = self.buffer[self.pos - 4..].as_ptr();
            unsafe { *(p as *const f32) }.swap_bytes()
        }
    }

    #[inline]
    pub fn read_f64(&mut self) -> f64 {
        self.pos += 8;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return 0f64;
        }
        #[cfg(target_endian = "little")]
        {
            let p: *const u8 = self.buffer[self.pos - 8..].as_ptr();
            unsafe { *(p as *const f64) }
        }
        #[cfg(not(target_endian = "little"))]
        {
            let p: *const u8 = self.buffer[self.pos - 8..].as_ptr();
            unsafe { *(p as *const f64) }.swap_bytes()
        }
    }

    #[inline]
    pub fn read_bytes(&mut self, size: usize) -> &[u8] {
        self.pos += size;
        if self.pos > self.buffer.len() {
            debug_assert!(false);
            return &[];
        }
        &self.buffer[(self.pos - size)..self.pos]
    }
}

#[inline]
pub fn write_u8(buffer: &mut [u8], n: u8) {
    if 1 > buffer.len() {
        debug_assert!(false);
        return ();
    }
    buffer[0] = n;
}

#[inline]
pub fn write_i8(buffer: &mut [u8], n: i8) {
    if 1 > buffer.len() {
        debug_assert!(false);
        return ();
    }
    buffer[0] = n as u8;
}

#[inline]
pub fn write_u16(buffer: &mut [u8], n: u16) {
    if 2 > buffer.len() {
        debug_assert!(false);
        return ();
    }
    let ns = n.to_le_bytes();
    buffer[0] = ns[0];
    buffer[1] = ns[1];
}
#[inline]
pub fn write_i16(buffer: &mut [u8], n: i16) {
    if 2 > buffer.len() {
        debug_assert!(false);
        return ();
    }
    let ns = n.to_le_bytes();
    buffer[0] = ns[0];
    buffer[1] = ns[1];
}

#[inline]
pub fn write_u32(buffer: &mut [u8], n: u32) {
    if 4 > buffer.len() {
        debug_assert!(false);
        return ();
    }
    let ns = n.to_le_bytes();
    buffer[0] = ns[0];
    buffer[1] = ns[1];
    buffer[2] = ns[2];
    buffer[3] = ns[3];
}
#[inline]
pub fn write_i32(buffer: &mut [u8], n: i32) {
    if 4 > buffer.len() {
        debug_assert!(false);
        return ();
    }
    let ns = n.to_le_bytes();
    buffer[0] = ns[0];
    buffer[1] = ns[1];
    buffer[2] = ns[2];
    buffer[3] = ns[3];
}
#[inline]
pub fn write_u64(buffer: &mut [u8], n: u64) {
    if 8 > buffer.len() {
        debug_assert!(false);
        return ();
    }
    let ns = n.to_le_bytes();
    buffer[0] = ns[0];
    buffer[1] = ns[1];
    buffer[2] = ns[2];
    buffer[3] = ns[3];
    buffer[4] = ns[4];
    buffer[5] = ns[5];
    buffer[6] = ns[6];
    buffer[7] = ns[7];
}

#[inline]
pub fn write_i64(buffer: &mut [u8], n: i64) {
    if 8 > buffer.len() {
        debug_assert!(false);
        return ();
    }
    let ns = n.to_le_bytes();
    buffer[0] = ns[0];
    buffer[1] = ns[1];
    buffer[2] = ns[2];
    buffer[3] = ns[3];
    buffer[4] = ns[4];
    buffer[5] = ns[5];
    buffer[6] = ns[6];
    buffer[7] = ns[7];
}

#[inline]
pub fn write_f32(buffer: &mut [u8], n: f32) {
    if 4 > buffer.len() {
        debug_assert!(false);
        return ();
    }
    let ns = n.to_le_bytes();
    buffer[0] = ns[0];
    buffer[1] = ns[1];
    buffer[2] = ns[2];
    buffer[3] = ns[3];
}

#[inline]
pub fn write_f64(buffer: &mut [u8], n: f64) {
    if 8 > buffer.len() {
        debug_assert!(false);
        return ();
    }
    let ns = n.to_le_bytes();
    buffer[0] = ns[0];
    buffer[1] = ns[1];
    buffer[2] = ns[2];
    buffer[3] = ns[3];
    buffer[4] = ns[4];
    buffer[5] = ns[5];
    buffer[6] = ns[6];
    buffer[7] = ns[7];
}

#[inline]
pub fn write_bytes(buffer: &mut [u8], n: &[u8]) {
    let len = n.len();
    if len == 0 {
        debug_assert!(false);
        return ();
    }
    if len > buffer.len() {
        debug_assert!(false);
        return ();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(n.as_ptr(), buffer[0..len].as_mut_ptr(), len);
    }
}

#[inline]
pub fn read_u8(buffer: &[u8]) -> u8 {
    if 1 > buffer.len() {
        debug_assert!(false);
        return 0u8;
    }
    buffer[0]
}
#[inline]
pub fn read_i8(buffer: &[u8]) -> i8 {
    if 1 > buffer.len() {
        debug_assert!(false);
        return 0i8;
    }
    buffer[0] as i8
}
#[inline]
pub fn read_u16(buffer: &[u8]) -> u16 {
    if 2 > buffer.len() {
        debug_assert!(false);
        return 0u16;
    }
    let p: *const u8 = buffer.as_ptr();
    #[cfg(target_endian = "little")]
    {
        unsafe { *(p as *const u16) }
    }
    #[cfg(not(target_endian = "little"))]
    {
        unsafe { *(p as *const u16) }.swap_bytes()
    }
}
#[inline]
pub fn read_i16(buffer: &[u8]) -> i16 {
    if 2 > buffer.len() {
        debug_assert!(false);
        return 0i16;
    }
    let p: *const u8 = buffer.as_ptr();
    #[cfg(target_endian = "little")]
    {
        unsafe { *(p as *const i16) }
    }
    #[cfg(not(target_endian = "little"))]
    {
        unsafe { *(p as *const i16) }.swap_bytes()
    }
}
#[inline]
pub fn read_u32(buffer: &[u8]) -> u32 {
    if 4 > buffer.len() {
        debug_assert!(false);
        return 0u32;
    }
    let p: *const u8 = buffer.as_ptr();
    #[cfg(target_endian = "little")]
    {
        unsafe { *(p as *const u32) }
    }
    #[cfg(not(target_endian = "little"))]
    {
        unsafe { *(p as *const u32) }.swap_bytes()
    }
}
#[inline]
pub fn read_i32(buffer: &[u8]) -> i32 {
    if 4 > buffer.len() {
        debug_assert!(false);
        return 0i32;
    }
    let p: *const u8 = buffer.as_ptr();
    #[cfg(target_endian = "little")]
    {
        unsafe { *(p as *const i32) }
    }
    #[cfg(not(target_endian = "little"))]
    {
        unsafe { *(p as *const i32) }.swap_bytes()
    }
}

#[inline]
pub fn read_u64(buffer: &[u8]) -> u64 {
    if 8 > buffer.len() {
        debug_assert!(false);
        return 0u64;
    }
    let p: *const u8 = buffer.as_ptr();
    #[cfg(target_endian = "little")]
    {
        unsafe { *(p as *const u64) }
    }
    #[cfg(not(target_endian = "little"))]
    {
        unsafe { *(p as *const u64) }.swap_bytes()
    }
}
#[inline]
pub fn read_i64(buffer: &[u8]) -> i64 {
    if 8 > buffer.len() {
        debug_assert!(false);
        return 0i64;
    }
    let p: *const u8 = buffer.as_ptr();
    #[cfg(target_endian = "little")]
    {
        unsafe { *(p as *const i64) }
    }
    #[cfg(not(target_endian = "little"))]
    {
        unsafe { *(p as *const i64) }.swap_bytes()
    }
}

#[inline]
pub fn read_f32(buffer: &[u8]) -> f32 {
    if 4 > buffer.len() {
        debug_assert!(false);
        return 0f32;
    }
    #[cfg(target_endian = "little")]
    {
        let p: *const u8 = buffer.as_ptr();
        unsafe { *(p as *const f32) }
    }
    #[cfg(not(target_endian = "little"))]
    {
        let p: *const u8 = buffer.as_ptr();
        unsafe { *(p as *const f32) }.swap_bytes()
    }
}

#[inline]
pub fn read_f64(buffer: &[u8]) -> f64 {
    if 8 > buffer.len() {
        debug_assert!(false);
        return 0f64;
    }
    #[cfg(target_endian = "little")]
    {
        let p: *const u8 = buffer.as_ptr();
        unsafe { *(p as *const f64) }
    }
    #[cfg(not(target_endian = "little"))]
    {
        let p: *const u8 = buffer.as_ptr();
        unsafe { *(p as *const f64) }.swap_bytes()
    }
}

#[inline]
pub fn read_bytes(buffer: &[u8]) -> Vec<u8> {
    let len = buffer.len();
    if len == 0 {
        debug_assert!(false);
        return vec![];
    }
    let mut result = vec![0u8; len];
    unsafe {
        std::ptr::copy_nonoverlapping(buffer.as_ptr(), result[0..len].as_mut_ptr(), len);
    }
    result
}

#[test]
fn test_bytes() {
    use crate::bytes;
    let mut buffer = vec![0u8; 16];
    let data_u16: u16 = 1024;
    let u8s = data_u16.to_le_bytes();
    bytes::write_bytes(&mut buffer, &u8s);
    debug_assert_eq!(bytes::read_u16(&buffer), data_u16);
    let data_u32: u32 = 999999;
    bytes::write_u32(&mut buffer, data_u32);
    debug_assert_eq!(bytes::read_u32(&buffer), data_u32);
    let data_u64: u64 = 999999999999999;
    bytes::write_u64(&mut buffer, data_u64);
    debug_assert_eq!(bytes::read_u64(&buffer), data_u64);
    let data_f32: f32 = 9999.999;
    bytes::write_f32(&mut buffer, data_f32);
    debug_assert_eq!(bytes::read_f32(&buffer), data_f32);
    let data_f64: f64 = 99999999999999999.999;
    bytes::write_f64(&mut buffer, data_f64);
    debug_assert_eq!(bytes::read_f64(&buffer), data_f64);
    let mut vbytes = vec![0u8; 42];
    let mut _bytes = bytes::Bytes::new(vbytes.as_mut_slice());
    _bytes.write_bytes("hello ccc".as_bytes());
    _bytes.set_pos(0);
    println!("str:{}", String::from_utf8_lossy(_bytes.read_bytes(8)));
    let mut vec = vec![0u8; 42];
    println!("Vec Len:{}", vec.len());
    let mut _bytes = bytes::Bytes::new(vec.as_mut_slice());
    println!("bytes size:{}", _bytes.get_size());
    _bytes.write_u8(9);
    _bytes.write_i8(-9);
    _bytes.write_u16(999);
    _bytes.write_i16(-999);
    _bytes.write_u32(99999);
    _bytes.write_i32(-99999);
    _bytes.write_u64(9999999999);
    _bytes.write_i64(-9999999999);
    _bytes.write_f32(99999.99);
    _bytes.write_f64(-99999.99);
    println!("pos:{}", _bytes.get_pos());
    _bytes.set_pos(0);
    println!("read_u8:{}", _bytes.read_u8());
    println!("read_i8:{}", _bytes.read_i8());
    println!("read_u16:{}", _bytes.read_u16());
    println!("read_i16:{}", _bytes.read_i16());
    println!("read_u32:{}", _bytes.read_u32());
    println!("read_i32:{}", _bytes.read_i32());
    println!("read_u64:{}", _bytes.read_u64());
    println!("read_i64:{}", _bytes.read_i64());
    println!("read_f32:{}", _bytes.read_f32());
    println!("read_f64:{}", _bytes.read_f64());
    println!("pos:{}", _bytes.get_pos());
    let mut f32_bytes = 99.9f32.to_le_bytes();
    let mut _bytes = bytes::Bytes::new(&mut f32_bytes);
    println!("leu8_to_f32:{}", _bytes.read_f32());
    let mut f64_bytes = 99.9f64.to_le_bytes();
    let mut _bytes = bytes::Bytes::new(&mut f64_bytes);
    println!("leu8_to_f64:{}", _bytes.read_f64());
    let mut be_bytes = 1u16.to_be_bytes();
    let mut le_bytes = 1u16.to_le_bytes();
    let mut bytes_be = bytes::Bytes::new(&mut be_bytes);
    let mut bytes_le = bytes::Bytes::new(&mut le_bytes);
    println!(
        "be:{} le:{}",
        bytes_be.read_u16().to_le(),
        bytes_le.read_u16()
    );
    let mut fle_bytes = 1f32.to_le_bytes();
    let mut fbe_bytes = 0.000000000000000000000000000000000000000046006f32.to_be_bytes();
    let mut bytes_fle = bytes::Bytes::new(&mut fle_bytes);
    let mut bytes_fbe = bytes::Bytes::new(&mut fbe_bytes);
    println!("be:{} le:{}", bytes_fle.read_f32(), bytes_fbe.read_f32());
}

/*
fn test() {
    let mut buffer = vec![0u8; 16];

    let data_u16: u16 = 1024;
    let u8s = data_u16.to_le_bytes();
    bytes::write_bytes(&mut buffer, &u8s);
    debug_assert_eq!(bytes::read_u16(&buffer), data_u16);

    let data_u32: u32 = 999999;
    bytes::write_u32(&mut buffer, data_u32);
    debug_assert_eq!(bytes::read_u32(&buffer), data_u32);

    let data_u64: u64 = 999999999999999;
    bytes::write_u64(&mut buffer, data_u64);
    debug_assert_eq!(bytes::read_u64(&buffer), data_u64);

    let data_f32: f32 = 9999.999;
    bytes::write_f32(&mut buffer, data_f32);
    debug_assert_eq!(bytes::read_f32(&buffer), data_f32);

    let data_f64: f64 = 99999999999999999.999;
    bytes::write_f64(&mut buffer, data_f64);
    debug_assert_eq!(bytes::read_f64(&buffer), data_f64);


    let mut vbytes = vec![0u8; 42];
    let mut _bytes = bytes::Bytes::new(vbytes.as_mut_slice());
    _bytes.write_bytes("hello ccc".as_bytes());
    _bytes.set_pos(0);
    println!("str:{}", String::from_utf8_lossy(_bytes.read_bytes(8)));
    let mut vec = vec![0u8; 42];
    println!("Vec Len:{}", vec.len());
    let mut _bytes = bytes::Bytes::new(vec.as_mut_slice());
    println!("bytes size:{}", _bytes.get_size());
    _bytes.write_u8(9);
    _bytes.write_i8(-9);
    _bytes.write_u16(999);
    _bytes.write_i16(-999);
    _bytes.write_u32(99999);
    _bytes.write_i32(-99999);
    _bytes.write_u64(9999999999);
    _bytes.write_i64(-9999999999);
    _bytes.write_f32(99999.99);
    _bytes.write_f64(-99999.99);
    println!("pos:{}", _bytes.get_pos());
    _bytes.set_pos(0);
    println!("read_u8:{}", _bytes.read_u8());
    println!("read_i8:{}", _bytes.read_i8());
    println!("read_u16:{}", _bytes.read_u16());
    println!("read_i16:{}", _bytes.read_i16());
    println!("read_u32:{}", _bytes.read_u32());
    println!("read_i32:{}", _bytes.read_i32());
    println!("read_u64:{}", _bytes.read_u64());
    println!("read_i64:{}", _bytes.read_i64());
    println!("read_f32:{}", _bytes.read_f32());
    println!("read_f64:{}", _bytes.read_f64());
    println!("pos:{}", _bytes.get_pos());
    let mut f32_bytes = 99.9f32.to_le_bytes();
    let mut _bytes = bytes::Bytes::new(&mut f32_bytes);
    println!("leu8_to_f32:{}", _bytes.read_f32());
    let mut f64_bytes = 99.9f64.to_le_bytes();
    let mut _bytes = bytes::Bytes::new(&mut f64_bytes);
    println!("leu8_to_f64:{}", _bytes.read_f64());
    let mut be_bytes = 1u16.to_be_bytes();
    let mut le_bytes = 1u16.to_le_bytes();
    let mut bytes_be = bytes::Bytes::new(&mut be_bytes);
    let mut bytes_le = bytes::Bytes::new(&mut le_bytes);
    println!(
        "be:{} le:{}",
        bytes_be.read_u16().to_le(),
        bytes_le.read_u16()
    );
    let mut fle_bytes = 1f32.to_le_bytes();
    let mut fbe_bytes = 0.000000000000000000000000000000000000000046006f32.to_be_bytes();
    let mut bytes_fle = bytes::Bytes::new(&mut fle_bytes);
    let mut bytes_fbe = bytes::Bytes::new(&mut fbe_bytes);
    println!("be:{} le:{}", bytes_fle.read_f32(), bytes_fbe.read_f32());

}

*/
