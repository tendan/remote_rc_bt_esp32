use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, Output};
use log::info;

use button::button_pressed_for;
use config::*;
use led::led_blink;

pub mod board;
mod button;
mod config;
mod led;
pub mod motor;

#[embassy_executor::task(pool_size = 1)]
pub async fn ble_activation_control(
    mut button: Input<'static>,
    mut indicator_led: Output<'static>,
    ble_advertisement_flag: &'static Signal<CriticalSectionRawMutex, bool>,
) -> ! {
    loop {
        button.wait_for_low().await;
        Timer::after(BUTTON_DEBOUNCE_DELAY).await;
        // For pull-up button
        indicator_led.set_high();
        info!("Pressed the button!");

        if button_pressed_for(BLE_BUTTON_HOLD_TIME, &mut button).await {
            ble_advertisement_flag.signal(true);
            led_blink(
                BLE_ADVERTISEMENT_TIME,
                BLE_LED_BLINK_PERIOD,
                &mut indicator_led,
            )
            .await;
            ble_advertisement_flag.signal(false);
        }

        info!("Released the button!");
        indicator_led.set_low();
    }
}
