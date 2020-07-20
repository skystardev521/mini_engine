use mysqlclient_sys as ffi;
use std::ffi::CStr;

pub type MysqlRow = ffi::MYSQL_ROW;
pub type MysqlRes = *mut ffi::MYSQL_RES;
pub type MysqlField = *mut ffi::MYSQL_FIELD;

pub struct QueryResult(MysqlRes);

impl Drop for QueryResult {
    fn drop(&mut self) {
        unsafe { ffi::mysql_free_result(self.0) }
    }
}

impl QueryResult {
    pub fn new(mysql_res: MysqlRes) -> Self {
        QueryResult(mysql_res)
    }
    #[inline]
    pub fn fetch_row(&self) -> MysqlRow {
        unsafe { ffi::mysql_fetch_row(self.0) }
    }
    #[inline]
    /// 结果集的列  field.name
    pub fn fetch_field(&self) -> MysqlField {
        unsafe { ffi::mysql_fetch_field(self.0) }
    }

    #[inline]
    /// 结果集的列数组  field[0].name
    pub fn fetch_fields(&self) -> MysqlField {
        unsafe { ffi::mysql_fetch_fields(self.0) }
    }

    #[inline]
    /// 字段的数量
    pub fn num_rows(&self) -> u64 {
        unsafe { ffi::mysql_num_rows(self.0) as u64 }
    }
    /// 结果的字段数量
    pub fn num_fields(&self) -> u32 {
        unsafe { ffi::mysql_num_fields(self.0) as u32 }
    }

    #[inline]
    /// 字段值的长度数组
    pub fn fetch_lengths(&self) -> *mut u64 {
        unsafe { ffi::mysql_fetch_lengths(self.0) as *mut u64 }
    }
}

trait Value<T> {
    fn data(&self, ptr: *const i8, size: usize) -> T;
}

impl Value<i8> for i8 {
    fn data(&self, ptr: *const i8, _size: usize) -> i8 {
        if ptr.is_null() {
            return *self;
        }
        let cs = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        if let Ok(val) = cs.parse::<i8>() {
            return val as i8;
        }
        return *self;
    }
}

impl Value<i16> for i16 {
    fn data(&self, ptr: *const i8, _size: usize) -> i16 {
        if ptr.is_null() {
            return *self;
        }
        let cs = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        if let Ok(val) = cs.parse::<i16>() {
            return val;
        }
        return *self;
    }
}

impl Value<i32> for i32 {
    fn data(&self, ptr: *const i8, _size: usize) -> i32 {
        if ptr.is_null() {
            return *self;
        }
        let cs = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        if let Ok(val) = cs.parse::<i32>() {
            return val;
        }
        return *self;
    }
}

impl Value<f64> for f64 {
    fn data(&self, ptr: *const i8, _size: usize) -> f64 {
        if ptr.is_null() {
            return *self;
        }
        let cs = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        if let Ok(val) = cs.parse::<f64>() {
            return val;
        }
        return *self;
    }
}

impl Value<Vec<i8>> for Vec<i8> {
    fn data(&self, ptr: *const i8, size: usize) -> Vec<i8> {
        if ptr.is_null() {
            return self.to_vec();
        }
        let mut buffer = vec![0i8; size];
        unsafe { std::ptr::copy_nonoverlapping(ptr, buffer[0..].as_mut_ptr(), size) }
        buffer
    }
}

impl Value<String> for String {
    fn data(&self, ptr: *const i8, _size: usize) -> String {
        if ptr.is_null() {
            return self.to_string();
        }
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    }
}

#[macro_export]
macro_rules! QR {
    ( $qr:expr, $( $f:expr ),* ) => {
        {
        let num_rows = $qr.num_rows() as usize;
        let num_fields = $qr.num_fields() as usize;
        let mut vec_result = Vec::with_capacity(num_rows);
        loop {
            let row = $qr.fetch_row();
            if row.is_null() {
                break;
            }
            let fetch_lengths = $qr.fetch_lengths() as *const u64;
            let val_size_array = ptr::slice_from_raw_parts(fetch_lengths, num_fields as usize);
            let row_val_array = ptr::slice_from_raw_parts(row, num_fields as usize);
            let mut idx = 0;
            vec_result.push(
                (
                $(
                    {
                        if idx == num_fields{
                            $f
                        }else {
                            let val = unsafe { &*row_val_array }[idx];
                            let val_size = unsafe { &*val_size_array }[idx];
                            idx += 1;
                            Value::data(&$f, val, val_size as usize)
                        }

                    },
                )*
                )
            );
        }
        vec_result
        }
    };
}

/*
#[inline]
fn c_str_to_string(ptr: *const i8) -> String {
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    //.into_owned()
}

#[inline]
fn c_str_to_vec(ptr: *const i8, size: usize) -> Vec<i8> {
    let mut buffer = vec![0i8; size];
    unsafe { std::ptr::copy_nonoverlapping(ptr, buffer[0..].as_mut_ptr(), size) }
    buffer
}
*/
