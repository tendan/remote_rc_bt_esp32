use crate::hardware::led::led_blink;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use esp_hal::gpio::{Input, Output};
use log::info;

mod led;
mod motor;

#[embassy_executor::task(pool_size = 1)]
pub async fn ble_activation_control(
    mut button: Input<'static>,
    mut indicator_led: Output<'static>,
    ble_advertisement_flag: &'static Signal<CriticalSectionRawMutex, bool>,
) -> ! {
    loop {
        // For pull-up button
        button.wait_for_falling_edge().await;
        indicator_led.set_high();
        info!("Pressed the button!");
        let mut millis_elapsed = 0;
        while button.is_low() {
            if millis_elapsed >= 3000 {
                ble_advertisement_flag.signal(true);
                led_blink(5000, 500, &mut indicator_led).await;
                ble_advertisement_flag.signal(false);
                break;
            }
            Timer::after_millis(1).await;
            millis_elapsed += 1;
        }
        indicator_led.set_low();
    }
}
