use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    watch::{Sender, Watch},
};
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    peripherals::{Peripherals, BT, TIMG0},
    timer::timg::TimerGroup,
};
use esp_radio::ble::controller::BleConnector;
use esp_radio::Controller;
use static_cell::StaticCell;
use trouble_host::prelude::ExternalController;

use crate::hardware::config::MotorsConfiguration;
use crate::hardware::motor::{
    BinaryAccelerator, BinaryMotor, BinarySteeringAxle, Motors, RobotChassis,
};

pub struct Board {
    pub ble_advertisement_button: Input<'static>,
    pub ble_indicator_led: Output<'static>,
    pub ble_controller: ExternalController<BleConnector<'static>, 20>,
    pub ble_advertisement_signal: &'static Watch<CriticalSectionRawMutex, bool, 2>,
    pub motors: MotorsConfiguration,
}

impl Board {
    pub fn init(peripherals: Peripherals) -> Self {
        init_embassy_runtime(peripherals.TIMG0);

        let input_conf = InputConfig::default().with_pull(esp_hal::gpio::Pull::Up);
        let ble_advertisement_button = Input::new(peripherals.GPIO4, input_conf);
        let ble_indicator_led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

        let radio = init_radio();
        let (ble_controller, ble_advertisement_signal) = init_bluetooth(peripherals.BT, radio);

        let accelerator = BinaryAccelerator {
            motor_forward: BinaryMotor {
                motor: Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default()),
            },
            motor_backward: BinaryMotor {
                motor: Output::new(peripherals.GPIO33, Level::Low, OutputConfig::default()),
            },
        };
        let steering = BinarySteeringAxle {
            motor_left: BinaryMotor {
                motor: Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default()),
            },
            motor_right: BinaryMotor {
                motor: Output::new(peripherals.GPIO26, Level::Low, OutputConfig::default()),
            },
        };
        let chassis = RobotChassis::new(accelerator, steering);
        let motors = Motors::setup(chassis);
        // let motors = Motors::setup(MotorSetup {
        //     accelerator: Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default()),
        //     backmove: Output::new(peripherals.GPIO33, Level::Low, OutputConfig::default()),
        //     steer_left: Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default()),
        //     steer_right: Output::new(peripherals.GPIO26, Level::Low, OutputConfig::default()),
        // });

        Self {
            ble_advertisement_button,
            ble_indicator_led,
            ble_controller,
            ble_advertisement_signal,
            motors,
        }
    }
}

fn init_radio() -> &'static mut Controller<'static> {
    static RADIO: StaticCell<Controller<'static>> = StaticCell::new();
    RADIO.init(esp_radio::init().unwrap())
}

fn init_bluetooth(
    bluetooth: BT<'static>,
    radio: &'static mut Controller<'static>,
) -> (
    ExternalController<BleConnector<'static>, 20>,
    &'static Watch<CriticalSectionRawMutex, bool, 2>,
) {
    static BLE_ADVERTISEMENT: Watch<CriticalSectionRawMutex, bool, 2> = Watch::new();

    let ble_advertisement_signal = &BLE_ADVERTISEMENT;

    let connector = BleConnector::new(radio, bluetooth);
    let ble_controller: ExternalController<_, 20> = ExternalController::new(connector);

    (ble_controller, ble_advertisement_signal)
}

fn init_embassy_runtime(timg0: TIMG0<'static>) {
    let timer0 = TimerGroup::new(timg0);

    esp_preempt::start(timer0.timer0);

    esp_hal_embassy::init(timer0.timer1);
}
