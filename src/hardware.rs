use embassy_futures::select::select;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Watch;
use embassy_time::Timer;
use esp_hal::gpio::{Input, Output};
use log::info;

use button::button_pressed_for;
use config::*;
use led::led_blink;

pub mod board;
mod button;
pub mod config;
mod led;
pub mod motor;

#[embassy_executor::task(pool_size = 1)]
pub async fn ble_activation_control(
    mut button: Input<'static>,
    mut indicator_led: Output<'static>,
    ble_advertisement_flag: &'static Watch<CriticalSectionRawMutex, bool, 2>,
) -> ! {
    loop {
        button.wait_for_low().await;
        Timer::after(BUTTON_DEBOUNCE_DELAY).await;
        // For pull-up button
        indicator_led.set_high();
        info!("Pressed the button!");

        if button_pressed_for(BLE_BUTTON_HOLD_TIME, &mut button).await {
            ble_advertisement_flag.sender().send(true);
            // ble_advertisement_flag.signal(true);
            let led_blinking = led_blink(
                BLE_ADVERTISEMENT_TIME,
                BLE_LED_BLINK_PERIOD,
                &mut indicator_led,
            );
            let external_disable = async {
                loop {
                    ble_advertisement_flag.receiver().unwrap().changed().await;
                    let new_state = ble_advertisement_flag.try_get().unwrap();
                    if !new_state {
                        break;
                    }
                    Timer::after_millis(50).await;
                }
            };
            select(led_blinking, external_disable).await;
            // ble_advertisement_flag.signal(false);
            ble_advertisement_flag.sender().send(false);
        }

        info!("Released the button!");
        indicator_led.set_low();
    }
}
