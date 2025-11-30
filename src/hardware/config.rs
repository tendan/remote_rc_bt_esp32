use crate::hardware::motor::{BinaryAccelerator, BinaryMotor, BinarySteeringAxle, Motors};
use embassy_time::Duration;

pub const BUTTON_DEBOUNCE_DELAY: Duration = Duration::from_millis(100);

pub const BLE_BUTTON_HOLD_TIME: Duration = Duration::from_secs(3);
pub const BLE_LED_BLINK_PERIOD: Duration = Duration::from_millis(500);
pub const BLE_ADVERTISEMENT_TIME: Duration = Duration::from_secs(5);

pub type MotorsConfiguration = Motors<
    BinaryAccelerator<BinaryMotor<'static>, BinaryMotor<'static>>,
    BinarySteeringAxle<BinaryMotor<'static>>,
>;
