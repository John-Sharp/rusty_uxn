use crate::emulators::uxn::device::{Device, MainRamInterface};
use chrono::{Local, Datelike, Timelike, DateTime};

pub struct DateTimeDevice {
    now_fn: fn() -> DateTime<Local>,    
}

impl DateTimeDevice {
    pub fn new() -> Self {
        DateTimeDevice{now_fn: Local::now}
    }
}

impl Device for DateTimeDevice {
    fn write(&mut self, port: u8, val: u8, main_ram: &mut dyn MainRamInterface) {
        if port > 0xf {
            panic!("attempting to write to port out of range");
        }

        match port {
            _ => {}
        }
    }

    fn read(&mut self, port: u8) -> u8 {
        if port > 0xf {
            panic!("attempting to read from port out of range");
        }

        let dt = (self.now_fn)();

        match port {
            0x0 => {
                let year = u16::try_from(dt.year()).unwrap();
                return year.to_be_bytes()[0];
            },
            0x1 => {
                let year = u16::try_from(dt.year()).unwrap();
                return year.to_be_bytes()[1];
            },
            0x2 => {
                return u8::try_from(dt.month0()).unwrap();
            },
            0x3 => {
                return u8::try_from(dt.day()).unwrap();
            },
            0x4 => {
                return u8::try_from(dt.hour()).unwrap();
            },
            0x5 => {
                return u8::try_from(dt.minute()).unwrap();
            },
            0x6 => {
                return u8::try_from(dt.second()).unwrap();
            },
            0x7 => {
                return u8::try_from(dt.weekday().num_days_from_sunday()).unwrap();
            },
            0x8 => {
                let year_day = u16::try_from(dt.ordinal0()).unwrap();
                return year_day.to_be_bytes()[0];
            },
            0x9 => {
                let year_day = u16::try_from(dt.ordinal0()).unwrap();
                return year_day.to_be_bytes()[1];
            },
            0xa => {
                // 'is daylight saving time', just return -1 (not known) in this
                // case
                return (-1_i8).to_be_bytes()[0];
            },
            _ => {
                return 0x0;
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulators::uxn::device::MainRamInterfaceError;
    use chrono::TimeZone;

    struct MockMainRamInterface {}

    impl MainRamInterface for MockMainRamInterface {
        fn read(&self, address: u16, num_bytes: u16) -> Result<Vec<u8>, MainRamInterfaceError> {
            panic!("should not be called");
        }

        fn write(&mut self, address: u16, bytes: &[u8]) -> Result<usize, MainRamInterfaceError> {
            panic!("should not be called");
        }
    }

    #[test]
    fn test_datetime() {
        let mut datetime_device = DateTimeDevice::new();
        datetime_device.now_fn = || { Local{}.datetime_from_str("1986-09-16 17:08:20", "%F %T").unwrap() };

        // test that the year returned matches that that the 'now_fn' returned
        let year_received = u16::from_be_bytes([
            datetime_device.read(0x0),
            datetime_device.read(0x1),
        ]);
        assert_eq!(1986, year_received);

        // test the month
        let month_received = datetime_device.read(0x2);
        // nb, month starts from 0
        assert_eq!(8, month_received);

        // test the day of the month
        let day_of_month_received = datetime_device.read(0x3);
        assert_eq!(16, day_of_month_received);

        // test the hour
        let hour_received = datetime_device.read(0x4);
        assert_eq!(17, hour_received);

        // test the minute
        let minute_received = datetime_device.read(0x5);
        assert_eq!(08, minute_received);

        // test the second
        let second_received = datetime_device.read(0x6);
        assert_eq!(20, second_received);

        // test the days since Sunday
        let week_day_received = datetime_device.read(0x7);
        assert_eq!(02, week_day_received);

        // test the days since January 1st
        let year_day_received = u16::from_be_bytes([
            datetime_device.read(0x8),
            datetime_device.read(0x9),
        ]);
        assert_eq!(258, year_day_received);

        // test whether dst (this is always reported -1 -- not awailable --
        // in this implementation)
        let is_dst_received = datetime_device.read(0xa);
        assert_eq!(255, is_dst_received);
    }
}
