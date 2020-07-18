use mysqlclient_sys as ffi;

pub type MysqlRow = ffi::MYSQL_ROW;
pub type MysqlRes = *mut ffi::MYSQL_RES;
pub type MysqlField = *mut ffi::MYSQL_FIELD;

pub type TINYINT = i8;
pub type SMALLINT = i16;
pub type INTEGER = i32;
pub type BIGINT = i64;
pub type FLOAT = f32;
pub type DOUBLE = f64;
pub type MyString = String;

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
