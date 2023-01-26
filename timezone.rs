// Thin Rust FFI layer around libtz's 'NETBSD_INSPIRED' functions.
//
// Copyright © 2023 David Caldwell <david@porkrind.org>
// License: MIT (see LICENSE.md file)

use libtz_sys::{TimezoneT, TimeT, tzalloc, tzfree, localtime_rz, mktime_z, posix2time_z, time2posix_z};
use std::ffi::CString;
use std::mem::MaybeUninit;
use crate::Tm;

/// A `Timezone` holds the storage for the libtz C library. Create one with
/// [`Timezone::new`] (to specify a specific timezone) or [`Timezone::default`]
/// (to use the default system timezone, which it looks for in
/// `/etc/localtime`).
///
/// # Example:
///
/// ```
/// use libtz::Timezone;
/// let tz = Timezone::new("America/Los_Angeles").expect("timezone alloc");
/// let time = 127810800;
/// let tm = tz.localtime(time).expect("localtime");
/// assert_eq!(tz.mktime(&tm).expect("mktime"), time);
/// ```
pub struct Timezone {
    tz: TimezoneT,
}

impl Timezone {
    /// Create a [`Timezone`] for the specified timezone name. The name can be
    /// something like `America/New_York`, `US/Pacific`, `UTC`, `PST`, etc. It
    /// can even specify a custom time conversion function. See
    /// [`libtz_sys::tzalloc`] for more details.
    pub fn new(name: &str) -> Result<Timezone, String> {
        let tzname = CString::new(name).map_err(|_| "name has internal null byte".to_string())?;
        let tz = unsafe { tzalloc(tzname.as_ptr()) };
        if tz == std::ptr::null_mut() {
            return Err("tzalloc failed".to_string());
        }
        Ok(Timezone{
            tz: tz
        })
    }

    /// Create a [`Timezone`] based on the `TZ` environment variable. If `TZ` is
    /// not set, use the tzfile stored in `/etc/localtime`. If that doesn't
    /// exist it will return an error.
    pub fn default() -> Result<Timezone, String> {
        use std::os::unix::ffi::OsStringExt;
        let zone_cstr;
        let zone = match std::env::var_os("TZ") {
            Some(zone) => { zone_cstr = CString::new(zone.into_vec()).map_err(|_| "name has internal null byte".to_string())?;
                            zone_cstr.as_ptr() },
            None       => std::ptr::null_mut(),
        };
        let tz = unsafe { tzalloc(zone) };
        if tz == std::ptr::null_mut() {
            return Err("tzalloc failed".to_string());
        }
        Ok(Timezone{
            tz: tz
        })
    }

    /// Convert system time to a local time [`Tm`].
    ///
    /// The `localtime` function corrects for the time zone and any time zone adjustments (such as Daylight
    /// Saving Time in the United States).
    pub fn localtime(&self, time: TimeT) -> Result<Tm, String> {
        let mut tztm = MaybeUninit::<libtz_sys::Tm>::uninit();
        let ret = unsafe { localtime_rz(self.tz, &time, tztm.as_mut_ptr()) };
        if ret == std::ptr::null_mut() {
            return Err(format!("errno={}", std::io::Error::last_os_error()));
        }
        let tztm = unsafe { tztm.assume_init() };
        Tm::try_from(&tztm)
    }

    /// Convert local time [`Tm`] to system time.
    ///
    /// The `mktime` function converts the broken-down time, expressed as local time, in the structure pointed
    /// to by `tm` into a calendar time value with the same encoding as that of the values returned by the
    /// `time` function.  The original values of the `tm_wday` and `tm_yday` components of the structure are
    /// ignored, and the original values of the other components are not restricted to their normal ranges.
    /// (A positive or zero value for `tm_isdst` causes `mktime` to presume initially that daylight saving
    /// time respectively, is or is not in effect for the specified time.
    ///
    /// A negative value for `tm_isdst` causes the `mktime` function to attempt to divine whether daylight
    /// saving time is in effect for the specified time; in this case it does not use a consistent rule and
    /// may give a different answer when later presented with the same argument.)  On successful completion,
    /// the values of the `tm_wday` and `tm_yday` components of the structure are set appropriately, and the
    /// other components are set to represent the specified calendar time, but with their values forced to
    /// their normal ranges; the final value of `tm_mday` is not set until `tm_mon` and `tm_year` are
    /// determined.  The `mktime` function returns the specified calendar time; If the calendar time cannot be
    /// represented, it returns an error.
    pub fn mktime(&self, tm: &Tm) -> Result<TimeT, String> {
        match unsafe { mktime_z(self.tz, &tm.into()) } {
            -1    => Err(format!("Invalid date specified")),
            time  => Ok(time),
        }
    }

    /// Convert from leap-second to POSIX `time_t`s.
    ///
    /// See [`libtz_sys::time2posix_z`] for details.
    pub fn time2posix(&self, time: TimeT) -> TimeT {
        unsafe { time2posix_z(self.tz, time) }
    }

    /// Convert from POSIX to leap-second `time_t`s.
    ///
    /// See [`libtz_sys::posix2time_z`] for details.
    pub fn posix2time(&self, time: TimeT) -> TimeT {
        unsafe { posix2time_z(self.tz, time) }
    }
}

impl Drop for Timezone {
    fn drop(&mut self) {
        unsafe { tzfree(self.tz) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let tz = Timezone::new("US/Pacific").expect("timezone alloc");
        let time = 946713600; // A strange falsetto sings...
        let tm = tz.localtime(time).expect("localtime");
        assert_eq!(tm, Tm{tm_sec    :0,
                          tm_min    :0,
                          tm_hour   :0,
                          tm_mday   :1,
                          tm_mon    :0,
                          tm_year   :100,
                          tm_wday   :6,
                          tm_yday   :0,
                          tm_isdst  :0,
                          tm_gmtoff :-28800,
                          tm_zone   :"PST".to_string()});
        assert_eq!(tz.mktime(&tm).expect("unix time"), time); // Round trip
    }

    #[test]
    fn default() {
        std::env::set_var("TZ", "Europe/Paris");
        let tz = Timezone::default().expect("load from TZ");
        let time = 915177600; // Tonight we're going to party...
        let tm;
        assert_eq!(tz.mktime({tm=tz.localtime(time).expect("localtime"); &tm}).expect("mktime"), time);
        assert_eq!(tm.tm_zone, "CET");

        std::env::remove_var("TZ");
        let tz = Timezone::default().expect("load from /etc/localtime");
        let time = 915177600; // Tonight we're going to party...
        assert_eq!(tz.mktime(&tz.localtime(time).expect("localtime")).expect("mktime"), time);
    }

    #[test]
    fn posix_conversions() {
        // The numbers in this test come from the libtz source explaining what
        // these functions do.
        let tz = Timezone::new("UTC").expect("timezone alloc");
        let posixtime: TimeT = 536457599;
        let time = tz.posix2time(posixtime);
        let tm = tz.localtime(time).expect("localtime");
        assert_eq!(tm, Tm{tm_sec    :59,
                          tm_min    :59,
                          tm_hour   :23,
                          tm_mday   :31,
                          tm_mon    :11,
                          tm_year   :86,
                          tm_wday   :3,
                          tm_yday   :364,
                          tm_isdst  :0,
                          tm_gmtoff :0,
                          tm_zone   :"UTC".to_string()});
        assert_eq!(tz.time2posix(time), posixtime); // Round Trip
    }
}
