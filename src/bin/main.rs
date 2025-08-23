#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::timer::timg::TimerGroup;
use esp_radio::Controller;
use log::info;
use static_cell::StaticCell;

// Local imports
use remote_rc_bt::hardware::ble_activation_control;
use remote_rc_bt::radio::ble_service;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    // let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timer0 = TimerGroup::new(peripherals.TIMG0);
    esp_radio_preempt_baremetal::init(timer0.timer0);

    static RADIO: StaticCell<Controller<'static>> = StaticCell::new();
    let radio = RADIO.init(esp_radio::init().unwrap());

    esp_hal_embassy::init(timer0.timer1);

    info!("Embassy initialized!");

    static BLE_ADVERTISEMENT: StaticCell<Signal<CriticalSectionRawMutex, bool>> = StaticCell::new();
    let ble_advertisement_signal = &*BLE_ADVERTISEMENT.init(Signal::new());

    let bluetooth = peripherals.BT;

    let input_conf = InputConfig::default().with_pull(esp_hal::gpio::Pull::Up);
    let button = Input::new(peripherals.GPIO4, input_conf);
    let indicator_led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    spawner
        .spawn(ble_activation_control(
            button,
            indicator_led,
            ble_advertisement_signal,
        ))
        .unwrap();
    spawner
        .spawn(ble_service(bluetooth, radio, ble_advertisement_signal))
        .unwrap();

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
