use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, WithTimeout};
use esp_hal::peripherals::BT;
use esp_radio::Controller;
use log::info;

use crate::radio::ble::{setup_ble, start_advertise};

mod ble;

#[embassy_executor::task(pool_size = 1)]
pub async fn ble_service(
    bluetooth_peripheral: BT<'static>,
    radio_controller: &'static Controller<'static>,
    ble_advertisement_flag: &'static Signal<CriticalSectionRawMutex, bool>,
) {
    info!("Started BLE service");
    // TODO: Setup BLE GATT Server
    setup_ble(bluetooth_peripheral, radio_controller).await;
    let ble_advertisement_timeout = Duration::from_millis(1000);
    loop {
        // TODO: BLE Event loop
        if let Ok(_) = ble_advertisement_flag
            .wait()
            .with_timeout(ble_advertisement_timeout)
            .await
        {
            start_advertise().await;
        }
    }
}
