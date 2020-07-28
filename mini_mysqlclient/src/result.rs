use log::error;
use mysqlclient_sys as ffi;
use std::ffi::CStr;
pub type MysqlRow = ffi::MYSQL_ROW;
pub type MysqlResult = ffi::MYSQL_RES;
pub type MysqlField = *mut ffi::MYSQL_FIELD;

pub struct QueryResult<T>(*mut T);

unsafe impl<T> Send for QueryResult<T> {}

impl<MysqlResult> Drop for QueryResult<MysqlResult> {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        unsafe { ffi::mysql_free_result(self.0 as *mut ffi::MYSQL_RES) }
    }
}

impl<MysqlResult> QueryResult<MysqlResult> {
    pub fn new(mysql_res: *mut MysqlResult) -> Self {
        QueryResult::<MysqlResult>(mysql_res)
    }
    #[inline]
    pub fn fetch_row(&self) -> MysqlRow {
        unsafe { ffi::mysql_fetch_row(self.0 as *mut ffi::MYSQL_RES) }
    }
    #[inline]
    #[allow(dead_code)]
    /// 结果集的列  field.name
    pub fn fetch_field(&self) -> MysqlField {
        unsafe { ffi::mysql_fetch_field(self.0 as *mut ffi::MYSQL_RES) }
    }

    #[inline]
    #[allow(dead_code)]
    /// 结果集的列数组  field[0].name
    pub fn fetch_fields(&self) -> MysqlField {
        unsafe { ffi::mysql_fetch_fields(self.0 as *mut ffi::MYSQL_RES) }
    }

    #[inline]
    /// 字段的数量
    pub fn num_rows(&self) -> u64 {
        unsafe { ffi::mysql_num_rows(self.0 as *mut ffi::MYSQL_RES) as u64 }
    }
    /// 结果的字段数量
    pub fn num_fields(&self) -> u32 {
        unsafe { ffi::mysql_num_fields(self.0 as *mut ffi::MYSQL_RES) as u32 }
    }

    #[inline]
    /// 字段值的长度数组
    pub fn fetch_lengths(&self) -> *mut u64 {
        unsafe { ffi::mysql_fetch_lengths(self.0 as *mut ffi::MYSQL_RES) as *mut u64 }
    }
}

pub trait CellValue<T> {
    fn parse_value(&self, ptr: *const i8, size: usize) -> T;
}

impl CellValue<i8> for i8 {
    #[inline]
    fn parse_value(&self, ptr: *const i8, size: usize) -> i8 {
        if ptr.is_null() || size == 0 {
            return *self;
        }
        let cs = c_str_to_string(ptr);
        match cs.parse::<i8>() {
            Ok(val) => val,
            Err(err) => {
                error!("mysql data->i32 err:{}", err);
                *self
            }
        }
    }
}

impl CellValue<i16> for i16 {
    #[inline]
    fn parse_value(&self, ptr: *const i8, size: usize) -> i16 {
        if ptr.is_null() || size == 0 {
            return *self;
        }
        let cs = c_str_to_string(ptr);
        match cs.parse::<i16>() {
            Ok(val) => val,
            Err(err) => {
                error!("mysql data->i32 err:{}", err);
                *self
            }
        }
    }
}

impl CellValue<i32> for i32 {
    #[inline]
    fn parse_value(&self, ptr: *const i8, size: usize) -> i32 {
        if ptr.is_null() || size == 0 {
            return *self;
        }
        let cs = c_str_to_string(ptr);
        match cs.parse::<i32>() {
            Ok(val) => val,
            Err(err) => {
                error!("mysql data->i32 err:{}", err);
                *self
            }
        }
    }
}

impl CellValue<f64> for f64 {
    #[inline]
    fn parse_value(&self, ptr: *const i8, size: usize) -> f64 {
        if ptr.is_null() || size == 0 {
            return *self;
        }
        let cs = c_str_to_string(ptr);
        match cs.parse::<f64>() {
            Ok(val) => val,
            Err(err) => {
                error!("mysql data->f64 err:{}", err);
                *self
            }
        }
    }
}

impl CellValue<Vec<i8>> for Vec<i8> {
    #[inline]
    fn parse_value(&self, ptr: *const i8, size: usize) -> Vec<i8> {
        if ptr.is_null() || size == 0 {
            self.to_vec()
        } else {
            c_str_to_vec(ptr, size)
        }
    }
}

impl CellValue<String> for String {
    #[inline]
    fn parse_value(&self, ptr: *const i8, size: usize) -> String {
        if ptr.is_null() || size == 0 {
            return self.to_string();
        } else {
            c_str_to_string(ptr)
        }
    }
}
#[inline]
fn c_str_to_string(ptr: *const i8) -> String {
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
}

#[inline]
fn c_str_to_vec(ptr: *const i8, size: usize) -> Vec<i8> {
    let mut buffer = vec![0i8; size];
    unsafe { std::ptr::copy_nonoverlapping(ptr, buffer[0..].as_mut_ptr(), size) }
    buffer
}

//#[feature(trace_macros)]
//trace_macros!(true);

#[macro_export]
macro_rules! query_result {
    ( $query_result:expr, $( $defaults:expr ),+ ) => {
        {
            let num_rows = $query_result.num_rows() as usize;
            if num_rows == 0{
                vec![]
            }else {

                let mut vec_result = Vec::with_capacity(num_rows);
                let num_fields = $query_result.num_fields() as usize;
                loop {
                    let row_data = $query_result.fetch_row();
                    if row_data.is_null() { break; }

                    let fetch_lengths = $query_result.fetch_lengths() as *const u64;
                    let row_data_slice = ptr::slice_from_raw_parts(row_data, num_fields as usize);
                    let data_size_slice = ptr::slice_from_raw_parts(fetch_lengths, num_fields as usize);

                    let mut _field: usize = 0;
                    let row_data_array = unsafe { &*row_data_slice };
                    let data_size_array = unsafe { &*data_size_slice };

                    vec_result.push(
                    (
                        $(
                            {
                                if _field == num_fields{
                                    $defaults
                                }else {
                                    let data = row_data_array[_field];
                                    let data_size = data_size_array[_field]; _field += 1;
                                    CellValue::parse_value(&$defaults, data, data_size as usize)
                                }
                            },
                        )+
                    ));
                }
                vec_result
            }
        }
    };
}
