#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::timer::timg::TimerGroup;
use esp_radio::ble::controller::BleConnector;
use esp_radio::Controller;
use log::info;
use trouble_host::prelude::*;

// use remote_rc_bt::hardware::motor::brake;
use remote_rc_bt::hardware::led::led_blink;
use remote_rc_bt::radio::ble::start_advertise;
use static_cell::StaticCell;
use trouble_host::HostResources;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.5.0

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

    let mut bluetooth = peripherals.BT;
    let connector = BleConnector::new(radio, bluetooth.reborrow());
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);

    // Demo task
    // let mut io = Io::new(peripherals.IO_MUX);
    // let output_config = OutputConfig::default()
    //     .with_drive_mode(DriveMode::PushPull)
    //     .with_pull(esp_hal::gpio::Pull::Down);
    let input_conf = InputConfig::default().with_pull(esp_hal::gpio::Pull::Up);
    let button = Input::new(peripherals.GPIO4, input_conf);
    let indicator_led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    spawner.spawn(button_pressed(button, indicator_led)).ok();

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}

#[embassy_executor::task]
async fn button_pressed(mut button: Input<'static>, mut indicator_led: Output<'static>) -> ! {
    loop {
        // For pull-up button
        button.wait_for_falling_edge().await;
        indicator_led.set_high();
        info!("Pressed the button!");
        let mut millis_elapsed = 0;
        while button.is_low() {
            if millis_elapsed >= 3000 {
                start_advertise().await;
                led_blink(5000, 500, &mut indicator_led).await;
                break;
            }
            Timer::after_millis(1).await;
            millis_elapsed += 1;
        }
        indicator_led.set_low();
    }
}
