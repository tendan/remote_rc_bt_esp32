use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, WithTimeout};
use log::info;

use crate::radio::ble::start_advertise;

mod ble;

#[embassy_executor::task(pool_size = 1)]
pub async fn ble_service(ble_advertisement_flag: &'static Signal<CriticalSectionRawMutex, bool>) {
    info!("Started BLE service");
    // TODO: Setup BLE GATT Server
    let ble_advertisement_timeout = Duration::from_millis(1000);
    loop {
        // TODO: BLE Event loop
        if let Ok(_) = ble_advertisement_flag
            .wait()
            .with_timeout(ble_advertisement_timeout)
            .await
        {
            start_advertise();
        }
    }
}
