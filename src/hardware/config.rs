use crate::hardware::motor::{
    BinaryMotor, DummySteeringAxle, LinearAcceleratorWithDirectionChoose, Motors, PwmMotor,
};
use embassy_time::Duration;
use esp_hal::peripherals::MCPWM0;

pub const BUTTON_DEBOUNCE_DELAY: Duration = Duration::from_millis(100);

pub const BLE_BUTTON_HOLD_TIME: Duration = Duration::from_secs(3);
pub const BLE_LED_BLINK_PERIOD: Duration = Duration::from_millis(500);
pub const BLE_ADVERTISEMENT_TIME: Duration = Duration::from_secs(5);

pub type MotorsConfiguration = Motors<
    LinearAcceleratorWithDirectionChoose<
        PwmMotor<'static, MCPWM0<'static>, 0>,
        BinaryMotor<'static>,
    >,
    DummySteeringAxle,
>;
