// Real Time Clock
// There is a standard clock on x86 machines that is battery backed from which
// we can attempt to retrive the current clock time.
//   see: https://wiki.osdev.org/RTC
// To use it though we need to interact with the CMOS. We need to write a
// register location to the `REGISTER_PORT` and then read a single byte from
// the `DATA_PORT`. Each resolution of the clock is on a different register.
// Additionally, the clock can be in 12 or 24 hour mode and the returned
// numbers can be in Binary or BCD mode. Binary is exactly what you'd expect
// while BCD mode is the digits as hex values so 43 seconds is `0x43` which
// needs to be converted to base 10.
//   see: https://wiki.osdev.org/CMOS#Getting_Current_Date_and_Time_from_RTC

use time::{Date, PrimitiveDateTime, Time};
use x86_64::structures::port::{PortRead, PortWrite};

///  Standard port for selecting which register to read or write.
const REGISTER_PORT: u16 = 0x70;
/// Standard port for reading or writing a byte after selecting a port.
const DATA_PORT: u16 = 0x71;

// Format for the `time` crate for conversion to string.
pub const DATE_TIME_FORMAT_EN: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

/// These addresses, which are conventionally colled Registers when interacting
/// with the CMOS, are where each byte of data lives.
enum TimeRegister {
    Second = 0x00,
    Minute = 0x02,
    Hour = 0x04,
    Day = 0x07,
    Month = 0x08,
    Year = 0x09,
    A = 0x0A,
    B = 0x0B,
}

/// A function to get the RTC configuration, all the data bytes and produce a
/// unix offset.
// TODO: this function can fail. Currently it panics, let's return a result and
// handle the error somewhere else.
pub fn read_rtc() -> PrimitiveDateTime {
    // The results of reading the CMOS could be out of sync if we don't wait for
    // it to signal that an update is finished.
    // TODO: we should actually wait for an update twice and read the values
    // after each to make sure we didn't get bad data.
    wait_til_not_updating();

    let register_b = unsafe { get_reg(TimeRegister::B) };
    let mut second = unsafe { get_reg(TimeRegister::Second) };
    let mut minute = unsafe { get_reg(TimeRegister::Minute) };
    let mut hour = unsafe { get_reg(TimeRegister::Hour) };
    let mut day = unsafe { get_reg(TimeRegister::Day) };
    let mut month = unsafe { get_reg(TimeRegister::Month) };
    let mut year = unsafe { get_reg(TimeRegister::Year) };

    if register_b & 0x04 == 0 {
        // If we get here the numbers are in BCD format and need to be converted
        // to base 10.
        second = (second & 0x0F) + ((second / 16) * 10);
        minute = (minute & 0x0F) + ((minute / 16) * 10);
        hour = ((hour & 0x0F) + (((hour & 0x70) / 16) * 10)) | (hour & 0x80);
        day = (day & 0x0F) + ((day / 16) * 10);
        month = (month & 0x0F) + ((month / 16) * 10);
        year = (year & 0x0F) + ((year / 16) * 10);
    }

    // The year returned is only 2 digits. This is a hacky way to turn that into
    // a full year. Should be made smarter somehow?
    let full_year: i32 = (2000 + (year as i32)).into();
    let calendar_month: time::Month = match month.try_into() {
        Ok(month) => month,
        _ => panic!("Could not convert {} to a calendar month", month),
    };

    let date = match Date::from_calendar_date(full_year, calendar_month, day) {
        Ok(date) => date,
        Err(err) => panic!("Could not create a Date {:?}", err),
    };

    let time = match Time::from_hms(hour, minute, second) {
        Ok(time) => time,
        Err(err) => panic!("Could not create a Time {:?}", err),
    };

    let date_time = date.with_time(time);

    date_time
}

unsafe fn get_reg(reg: TimeRegister) -> u8 {
    PortWrite::write_to_port(REGISTER_PORT, reg as u8);
    PortRead::read_from_port(DATA_PORT)
}

/// The CMOS update can be slow and if we read the registers before an update
/// cycles we may get bad, out of sync, data.
fn wait_til_not_updating() {
    loop {
        let byte = unsafe { get_reg(TimeRegister::A) };
        if byte & 0b01000000 == 0 {
            return;
        }
        core::hint::spin_loop();
    }
}
