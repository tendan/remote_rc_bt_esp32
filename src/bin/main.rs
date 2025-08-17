#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use bt_hci::controller::ExternalController;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{AnyPin, DriveMode, Input, InputConfig, Io, Level, Output, OutputConfig};
use esp_hal::interrupt::software::{SoftwareInterrupt, SoftwareInterruptControl};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::xtensa_lx_rt::interrupt;
use esp_hal_embassy::InterruptExecutor;
use esp_wifi::ble::controller::BleConnector;
use log::info;

use remote_rc_bt::hardware::motor::brake;

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
    // generator version: 0.5.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    // let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    // let timer1 = TimerGroup::new(peripherals.TIMG0);
    // let wifi_init =
    //     esp_wifi::init(timer1.timer0, rng).expect("Failed to initialize WIFI/BLE controller");
    // // find more examples https://github.com/embassy-rs/trouble/tree/main/examples/esp32
    // let transport = BleConnector::new(&wifi_init, peripherals.BT);
    // let _ble_controller = ExternalController::<_, 20>::new(transport);

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
        //info!("Pressed the button!");
        let mut millis_elapsed = 0;
        while button.is_low() {
            if millis_elapsed >= 3000 {
                // info!("Button held for 3 seconds!");
                led_blink(5000, 500, &mut indicator_led).await;
                break;
            }
            Timer::after_millis(1).await;
            millis_elapsed += 1;
        }
        indicator_led.set_low();
    }
}

async fn led_blink(mut time_ms: u64, period_ms: u64, led: &mut Output<'static>) {
    while time_ms > 0 {
        led.toggle();
        Timer::after_millis(period_ms).await;
        time_ms -= period_ms;
    }
}
