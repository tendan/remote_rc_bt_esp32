use esp_hal::peripherals::BT;
use esp_radio::ble::controller::BleConnector;
use esp_radio::Controller;
use log::info;
use trouble_host::prelude::*;
use trouble_host::HostResources;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

pub async fn setup_ble(
    mut bluetooth_peripheral: BT<'static>,
    radio_controller: &'static Controller<'static>,
) {
    let connector = BleConnector::new(radio_controller, bluetooth_peripheral.reborrow());
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
}

pub async fn start_advertise() {
    // TODO: Advertise
    info!("Begin advertising!");
}
