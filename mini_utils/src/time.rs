use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const YEAR_HOUR: u64 = 365 * 24;
const FOUR_YERA: u64 = 1461 * 24;

const MIN_MS: u64 = 60 * 1000;
const HOUR_MS: u64 = 3600 * 1000;
const DAY_MS: u64 = 24 * 3600 * 1000;
const YEAR_MS: u64 = 365 * 24 * 3600 * 1000;

const DAYS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

const MON_YDAY: [[u16; 12]; 2] = [
    [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334],
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335],
];

#[derive(Debug)]
pub struct Time {
    pub ms: u16,    /* 毫秒 – 取值区间为[0,999] */
    pub sec: u16,   /* 秒 – 取值区间为[0,59] */
    pub min: u16,   /* 分 - 取值区间为[0,59] */
    pub hour: u16,  /* 时 - 取值区间为[0,23] */
    pub day: u16,   /* 天 - 取值区间为[1,31] */
    pub month: u16, /* 月 - 取值区间为[0,11] */
    pub year: u16,  /* 年 - 其值等于实际年份减去1970 */
}

#[inline]
pub fn timestamp() -> u64 {
    let now = SystemTime::now();
    match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

#[inline]
/// timezone：单位小时
pub fn now_time(timezone: f32) -> Time {
    ts_to_time(timestamp() + (timezone * 60f32 * 60f32 * 1000f32) as u64)
}

pub fn ts_to_time(timestamp: u64) -> Time {
    let mut time: Time = Time {
        ms: 0,
        sec: 0,
        min: 0,
        hour: 0,
        day: 0,
        month: 0,
        year: 0,
    };

    let mut ts = timestamp;
    time.ms = (ts % 1000) as u16;
    ts /= 1000;

    //取秒时间
    time.sec = (ts % 60) as u16;
    ts /= 60;
    //取分钟时间
    time.min = (ts % 60) as u16;
    ts /= 60;
    //取过去多少个四年，每四年有 1461*24 小时
    let pass4year = (ts / FOUR_YERA) as u32;
    //计算年份
    time.year = (pass4year << 2) as u16 + 1970;

    //四年中剩下的小时数
    ts %= 1461 * 24;

    //校正闰年影响的年份，计算一年中剩下的小时数
    loop {
        //一年的小时数
        let mut hours_per_year = YEAR_HOUR;
        //判断闰年
        if (time.year & 3) == 0 {
            //是闰年，一年则多24小时，即一天
            hours_per_year += 24;
        }
        if ts < hours_per_year {
            break;
        }
        time.year += 1;
        ts -= hours_per_year;
    }
    //小时数
    time.hour = (ts % 24) as u16;
    //一年中剩下的天数
    ts /= 24;
    //假定为闰年
    ts += 1;
    //校正闰年的误差，计算月份，日期
    if (time.year & 3) == 0 {
        if ts > 60 {
            ts -= 1;
        } else {
            if ts == 60 {
                time.month = 1;
                time.day = 29;
                return time;
            }
        }
    }

    //计算月日
    loop {
        let day: u64 = DAYS[time.month as usize] as u64;
        if day < ts {
            ts -= day;
            time.month += 1;
        } else {
            break;
        };
    }
    time.day = ts as u16;
    time
}

pub fn time_to_ts(time: &Time) -> u64 {
    let mon_yday = if (time.year) % 4 == 0 && ((time.year) % 100 != 0 || (time.year) % 400 == 0) {
        MON_YDAY[1][(time.month - 1) as usize]
    } else {
        MON_YDAY[0][(time.month - 1) as usize]
    };

    // 以平年时间计算的秒数
    let mut timestamp: u64 = (time.ms as u64
        + time.sec as u64 * 1000
        + (time.min as u64 * MIN_MS)
        + (time.hour as u64 * HOUR_MS)
        + ((mon_yday as u64 + time.day as u64 - 1) * DAY_MS)
        + ((time.year as u64 - 1970) * YEAR_MS)) as u64;

    // 加上闰年的秒数
    for year in 1970..time.year {
        if (year) % 4 == 0 && ((year) % 100 != 0 || (year) % 400 == 0) {
            timestamp += DAY_MS;
        }
    }
    timestamp
}
