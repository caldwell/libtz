// Thin Rust FFI layer around libtz
//
// Copyright © 2023 David Caldwell <david@porkrind.org>
// License: MIT (see LICENSE.md file)

#![doc = include_str!("README.md")]

mod timezone;
pub use timezone::Timezone;

pub use libtz_sys::TimeT;
use std::mem::MaybeUninit;
use std::ffi::{CStr};

/// A broken down time representation, logically equivalent to `struct tm` in
/// unix (though not binary compatible).
///
/// Reference: <https://pubs.opengroup.org/onlinepubs/7908799/xsh/time.h.html>
#[derive(Clone, Debug, PartialEq)]
pub struct Tm {
    /** Seconds          [0, 60] */                 pub tm_sec    : i32,
    /** Minutes          [0, 59] */                 pub tm_min    : i32,
    /** Hour             [0, 23] */                 pub tm_hour   : i32,
    /** Day of the month [1, 31] */                 pub tm_mday   : i32,
    /** Month            [0, 11]  (January = 0) */  pub tm_mon    : i32,
    /** Year minus 1900 */                          pub tm_year   : i32,
    /** Day of the week  [0, 6]   (Sunday = 0) */   pub tm_wday   : i32,
    /** Day of the year  [0, 365] (Jan/01 = 0) */   pub tm_yday   : i32,
    /** Daylight savings flag */                    pub tm_isdst  : i32,

    /** Seconds East of UTC */                      pub tm_gmtoff : i64,
    /** Timezone abbreviation */                    pub tm_zone   : String,
}

impl TryFrom<&libtz_sys::Tm> for Tm {
    type Error = String;
    fn try_from(tztm: &libtz_sys::Tm) -> Result<Self, Self::Error> {
        let zone: &str = unsafe { CStr::from_ptr(tztm.tm_zone).to_str().map_err(|_| "Invalid utf8")? };

        Ok(Tm{
            tm_sec    : tztm.tm_sec,
            tm_min    : tztm.tm_min,
            tm_hour   : tztm.tm_hour,
            tm_mday   : tztm.tm_mday,
            tm_mon    : tztm.tm_mon,
            tm_year   : tztm.tm_year,
            tm_wday   : tztm.tm_wday,
            tm_yday   : tztm.tm_yday,
            tm_isdst  : tztm.tm_isdst,
            tm_gmtoff : tztm.tm_gmtoff,
            tm_zone   : zone.to_string(),
        })
    }
}

impl Into<libtz_sys::Tm> for &Tm {
    fn into(self) -> libtz_sys::Tm {
            libtz_sys::Tm{
                tm_sec    : self.tm_sec,
                tm_min    : self.tm_min,
                tm_hour   : self.tm_hour,
                tm_mday   : self.tm_mday,
                tm_mon    : self.tm_mon,
                tm_year   : self.tm_year,
                tm_wday   : self.tm_wday,
                tm_yday   : self.tm_yday,
                tm_isdst  : self.tm_isdst,
                tm_gmtoff : self.tm_gmtoff,
                tm_zone   : std::ptr::null_mut(),
        }
    }
}

/// Convert UTC [`Tm`] to system time.
///
/// This function is like [`Timezone::mktime()`][timezone::Timezone::mktime] except that it treats the `tm` as
/// UTC (ignoring the `tm_idst` and `tm_zone` members).
pub fn timegm(tm: &Tm) -> Result<TimeT, String> {
    match unsafe { libtz_sys::timegm(&tm.into()) } {
        -1    => Err(format!("Invalid date specified")),
        time  => Ok(time),
    }
}

/// Convert system time to UTC [`Tm`].
///
/// The `gmtime` function converts to Coordinated Universal Time, returning a pointer to a [`Tm`]
/// structure.
pub fn gmtime(time: TimeT) -> Result<Tm, String> {
    let mut tztm = MaybeUninit::<libtz_sys::Tm>::uninit();
    let ret = unsafe { libtz_sys::gmtime_r(&time, tztm.as_mut_ptr()) };
    if ret == std::ptr::null_mut() {
        return Err(format!("errno={}", std::io::Error::last_os_error()));
    }
    let tztm = unsafe { tztm.assume_init() };
    Tm::try_from(&tztm)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gmtime_test() {
        let time = 283996800;
        let tm = gmtime(time).expect("gmtime");
        assert_eq!(tm, Tm{tm_sec    :0,
                          tm_min    :0,
                          tm_hour   :0,
                          tm_mday   :1,
                          tm_mon    :0,
                          tm_year   :79,
                          tm_wday   :1,
                          tm_yday   :0,
                          tm_isdst  :0,
                          tm_gmtoff :0,
                          tm_zone   :"UTC".to_string()});
        assert_eq!(timegm(&tm).expect("timegm"), time);
    }

    #[test]
    fn test_readme_deps() {
        version_sync::assert_markdown_deps_updated!("README.md");
    }
}
