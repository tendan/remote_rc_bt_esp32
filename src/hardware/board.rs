use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use esp_hal::mcpwm::operator::PwmPinConfig;
use esp_hal::mcpwm::timer::PwmWorkingMode;
use esp_hal::mcpwm::McPwm;
use esp_hal::mcpwm::PeripheralClockConfig;
use esp_hal::time::Rate;
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    peripherals::{Peripherals, BT, TIMG0},
    timer::timg::TimerGroup,
};
use esp_radio::ble::controller::BleConnector;
use esp_radio::Controller;
use log::error;
use static_cell::StaticCell;
use trouble_host::prelude::ExternalController;

use crate::hardware::config::MotorsConfiguration;
use crate::hardware::motor::{
    BinaryMotor, DummySteeringAxle, LinearAcceleratorWithDirectionChoose, Motors, PwmMotor,
    RobotChassis,
};

pub struct Board {
    pub ble_advertisement_button: Input<'static>,
    pub ble_indicator_led: Output<'static>,
    pub ble_controller: ExternalController<BleConnector<'static>, 20>,
    pub ble_advertisement_signal: &'static Watch<CriticalSectionRawMutex, bool, 2>,
    pub motors: MotorsConfiguration,
}

impl Board {
    pub async fn init(peripherals: Peripherals) -> Self {
        init_embassy_runtime(peripherals.TIMG0);

        let input_conf = InputConfig::default().with_pull(esp_hal::gpio::Pull::Up);
        let ble_advertisement_button = Input::new(peripherals.GPIO4, input_conf);
        let ble_indicator_led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

        let radio = init_radio();
        let (ble_controller, ble_advertisement_signal) = init_bluetooth(peripherals.BT, radio);

        let clock_cfg = match PeripheralClockConfig::with_frequency(Rate::from_mhz(80)) {
            Ok(cfg) => cfg,
            Err(err) => {
                error!("Something went wrong with peripheral clock: {:?}", err);
                loop {}
            }
        };

        let mcpwm = McPwm::new(peripherals.MCPWM0, clock_cfg);

        let (mut timer0, mut operator0) = (mcpwm.timer0, mcpwm.operator0);

        operator0.set_timer(&timer0);

        let period_ticks = 100;
        let frequency_khz = 20;

        let timer_clock_cfg = match clock_cfg.timer_clock_with_frequency(
            period_ticks - 1,
            PwmWorkingMode::Increase,
            Rate::from_khz(frequency_khz),
        ) {
            Ok(cfg) => cfg,
            Err(err) => {
                error!("Something went wrong with timer clock: {:?}", err);
                loop {}
            }
        };
        timer0.start(timer_clock_cfg);

        // MCPWM gets dropped when it gets out of scope and disables timer which occures panic when trying to set timestamp for PWM
        //static TIMER: StaticCell<esp_hal::mcpwm::timer::Timer<0, MCPWM0<'_>>> = StaticCell::new();
        //let _timer_ref = TIMER.init(timer0);

        let accelerator_pin =
            operator0.with_pin_a(peripherals.GPIO33, PwmPinConfig::UP_ACTIVE_HIGH);
        let direction_selector_pin =
            Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default());

        let accelerator = LinearAcceleratorWithDirectionChoose {
            motor: PwmMotor::new(accelerator_pin, period_ticks),
            direction_selector: BinaryMotor::new(direction_selector_pin), //motor_backward: PwmMotor::new(&shared_pca, pwm_pca9685::Channel::C1),
        };

        let steering = DummySteeringAxle::new();

        let chassis = RobotChassis::new(accelerator, steering);
        let motors = Motors::setup(chassis);

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
