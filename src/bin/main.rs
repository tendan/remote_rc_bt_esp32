#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;

use remote_rc_bt::init::init_core_system;

// Local imports
use remote_rc_bt::hardware::{ble_activation_control, board::Board};
use remote_rc_bt::radio::start_ble;

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
    let peripherals = init_core_system();

    let board = Board::init(peripherals);

    spawner
        .spawn(ble_activation_control(
            board.ble_advertisement_button,
            board.ble_indicator_led,
            board.ble_advertisement_signal,
        ))
        .unwrap();

    let motors = board.motors;

    start_ble(board.ble_controller, board.ble_advertisement_signal, motors).await;
}
