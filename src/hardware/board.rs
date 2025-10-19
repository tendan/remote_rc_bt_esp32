use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    peripherals::{Peripherals, BT},
};
use esp_radio::ble::controller::BleConnector;
use esp_radio::Controller;
use static_cell::StaticCell;
use trouble_host::prelude::ExternalController;

use crate::init::init_embassy_runtime;

pub struct Board<'d, T> {
    pub ble_advertisement_button: Input<'d>,
    pub ble_indicator_led: Output<'d>,
    pub ble_controller: ExternalController<T, 20>,
    pub ble_advertisement_signal: &'static Signal<CriticalSectionRawMutex, bool>,
}

impl<'d> Board<'d, BleConnector<'d>> {
    fn init_radio() -> &'static mut Controller<'d> {
        static RADIO: StaticCell<Controller<'static>> = StaticCell::new();
        RADIO.init(esp_radio::init().unwrap())
    }
    fn init_bluetooth(
        bluetooth: BT<'static>,
        radio: &'static mut Controller<'static>,
    ) -> (
        ExternalController<BleConnector<'d>, 20>,
        &'d Signal<CriticalSectionRawMutex, bool>,
    ) {
        static BLE_ADVERTISEMENT: StaticCell<Signal<CriticalSectionRawMutex, bool>> =
            StaticCell::new();
        let ble_advertisement_signal = &*BLE_ADVERTISEMENT.init(Signal::new());

        let connector = BleConnector::new(radio, bluetooth);
        let ble_controller: ExternalController<_, 20> = ExternalController::new(connector);

        (ble_controller, ble_advertisement_signal)
    }
    pub fn init(peripherals: Peripherals) -> Self {
        init_embassy_runtime(peripherals.TIMG0);

        let input_conf = InputConfig::default().with_pull(esp_hal::gpio::Pull::Up);
        let ble_advertisement_button = Input::new(peripherals.GPIO4, input_conf);
        let ble_indicator_led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

        let radio = Board::init_radio();
        let (ble_controller, ble_advertisement_signal) =
            Board::init_bluetooth(peripherals.BT, radio);

        Self {
            ble_advertisement_button,
            ble_indicator_led,
            ble_controller,
            ble_advertisement_signal,
        }
    }
}
