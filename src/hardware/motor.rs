use crate::control::instruction::AddressablePeripheral;
use crate::control::instruction::PerformFunctionError;
use esp_hal::gpio::Level;
use esp_hal::gpio::Output;
use esp_hal::mcpwm::operator::PwmPin;
use esp_hal::mcpwm::PwmPeripheral;
use log::info;

pub trait MotorDriver {
    fn set_power(&mut self, value: u8);
    fn get_power(&self) -> u8;
}

pub enum MotorVariant<B, P> {
    Binary(B),
    Pwm(P),
}

impl<B, P> MotorDriver for MotorVariant<B, P>
where
    B: MotorDriver,
    P: MotorDriver,
{
    fn set_power(&mut self, power: u8) {
        match self {
            Self::Binary(b) => b.set_power(power),
            Self::Pwm(p) => p.set_power(power),
        }
    }
    fn get_power(&self) -> u8 {
        match self {
            Self::Binary(b) => b.get_power(),
            Self::Pwm(p) => p.get_power(),
        }
    }
}

// =============================
pub struct BinaryMotor<'a> {
    pub motor: Output<'a>,
}

impl<'a> BinaryMotor<'a> {
    pub fn new(motor: Output<'a>) -> Self {
        Self { motor }
    }
}

// pub type I2cPca<'a> = Pca9685<I2c<'a, Blocking>>;

// pub struct I2cPwmMotor<'a> {
//     pub driver: &'a RefCell<I2cPca<'a>>,
//     pub channel: Channel,
//     last_value: i8,
// }

pub struct PwmMotor<'a, PWM, const OP: u8> {
    pwm_motor: PwmPin<'a, PWM, OP, true>,
    period_ticks: u16,
}

impl<'a> MotorDriver for BinaryMotor<'a> {
    fn set_power(&mut self, value: u8) {
        self.motor.set_level(Level::from(value != 0));
    }
    fn get_power(&self) -> u8 {
        match self.motor.output_level() {
            Level::Low => 0,
            Level::High => 1,
        }
    }
}

// impl<'a> I2cPwmMotor<'a> {
//     pub fn new(driver: &'a RefCell<I2cPca<'a>>, channel: Channel) -> Self {
//         Self {
//             driver,
//             channel,
//             last_value: 0,
//         }
//     }
// }

fn map_range(val: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> u16 {
    let val_clamped = val.max(in_min).min(in_max);
    let numerator = (val_clamped - in_min) * (out_max - out_min);
    let denominator = in_max - in_min;
    (out_min + (numerator / denominator)) as u16
}

// impl<'a> MotorDriver for I2cPwmMotor<'a> {
//     fn set_power(&mut self, value: u8) {
//         info!("PWM Motor here!");
//         self.last_value = value;
//         // TODO: Replace magic number
//         let pwm_value = map_range(value as i32, -128, 127, 205, 410);
//         let mut pca = self.driver.borrow_mut();
//         pca.set_channel_on_off(self.channel, 0, pwm_value).ok();
//     }
//     fn get_power(&self) -> u8 {
//         self.last_value
//     }
// }

impl<'a, PWM: PwmPeripheral, const OP: u8> PwmMotor<'a, PWM, OP> {
    pub fn new(pwm_pin: PwmPin<'a, PWM, OP, true>, period_ticks: u16) -> Self {
        Self {
            pwm_motor: pwm_pin,
            period_ticks,
        }
    }

    pub fn set_duty_cycle(&mut self, percent: u8) {
        let percent = percent.min(100);

        let timestamp = (percent as u32 * self.period_ticks as u32) / 100;
        let timestamp = timestamp.min(self.period_ticks as u32) as u16;
        self.pwm_motor.set_timestamp(timestamp as u16);
    }

    pub fn get_duty_cycle(&self) -> u8 {
        ((self.pwm_motor.timestamp() as u32) * 100 / self.period_ticks as u32) as u8
    }
}

impl<'a, PWM: PwmPeripheral, const OP: u8> MotorDriver for PwmMotor<'a, PWM, OP> {
    fn set_power(&mut self, value: u8) {
        let duty_cycle = map_range(value as i32, 0, 255, 0, 100) as u8;
        self.set_duty_cycle(duty_cycle);
    }
    fn get_power(&self) -> u8 {
        let duty_cycle = self.get_duty_cycle() as i32;
        map_range(duty_cycle, 0, 100, 0, 255) as u8
    }
}

pub trait SteeringAxle {
    fn set_steering(&mut self, value: i8);
    fn get_current_steering(&self) -> i8;
}

pub trait Accelerator {
    fn set_throttle(&mut self, value: i8);
    fn get_current_throttle(&self) -> i8;
}

pub struct DummySteeringAxle {}

impl DummySteeringAxle {
    pub fn new() -> Self {
        Self {}
    }
}

impl SteeringAxle for DummySteeringAxle {
    #[allow(unused_variables)]
    fn set_steering(&mut self, value: i8) {}
    fn get_current_steering(&self) -> i8 {
        0
    }
}

pub struct BinarySteeringAxle<M>
where
    M: MotorDriver,
{
    pub motor_left: M,
    pub motor_right: M,
}

impl<M: MotorDriver> SteeringAxle for BinarySteeringAxle<M> {
    fn set_steering(&mut self, value: i8) {
        if value > 0 {
            self.motor_right.set_power(255);
            self.motor_left.set_power(0);
        } else if value < 0 {
            self.motor_left.set_power(255);
            self.motor_right.set_power(0);
        } else {
            self.motor_left.set_power(0);
            self.motor_right.set_power(0);
        }
    }
    fn get_current_steering(&self) -> i8 {
        if self.motor_right.get_power() == 255 {
            127
        } else if self.motor_left.get_power() == 0 {
            -128
        } else {
            0
        }
    }
}

pub struct ServoSteeringAxle<M> {
    pub motor_steer: M,
}

impl<M: MotorDriver> SteeringAxle for ServoSteeringAxle<M> {
    fn set_steering(&mut self, value: i8) {
        self.motor_steer
            .set_power(map_range(value as i32, -128, 127, 0, 255) as u8);
    }
    fn get_current_steering(&self) -> i8 {
        map_range(self.motor_steer.get_power() as i32, 0, 255, -128, 127) as i8
    }
}

pub struct BinaryAccelerator<MF, MB> {
    pub motor_forward: MF,
    pub motor_backward: MB,
}

impl<MF: MotorDriver, MB: MotorDriver> Accelerator for BinaryAccelerator<MF, MB> {
    fn set_throttle(&mut self, value: i8) {
        if value > 0 {
            self.motor_forward.set_power(127);
            self.motor_backward.set_power(0);
        } else if value < 0 {
            self.motor_backward.set_power(127);
            self.motor_forward.set_power(0);
        } else {
            self.motor_forward.set_power(0);
            self.motor_backward.set_power(0);
        }
    }
    fn get_current_throttle(&self) -> i8 {
        if self.motor_forward.get_power() == 1 {
            1
        } else if self.motor_backward.get_power() == 1 {
            -1
        } else {
            0
        }
    }
}

pub struct LinearAccelerator<M> {
    pub motor: M,
}

impl<M: MotorDriver> Accelerator for LinearAccelerator<M> {
    fn set_throttle(&mut self, value: i8) {
        self.motor
            .set_power(map_range(value as i32, -128, 127, 0, 255) as u8);
    }
    fn get_current_throttle(&self) -> i8 {
        map_range(self.motor.get_power() as i32, 0, 255, -128, 127) as i8
    }
}

pub struct LinearAcceleratorWithDirectionChoose<MA, MD> {
    pub motor: MA,
    pub direction_selector: MD,
}

impl<'a, MA: MotorDriver> Accelerator
    for LinearAcceleratorWithDirectionChoose<MA, BinaryMotor<'a>>
{
    fn set_throttle(&mut self, value: i8) {
        if value > 0 {
            self.direction_selector.motor.set_high();
            self.motor
                .set_power(map_range(value as i32, 1, 127, 0, 255) as u8);
        } else if value < 0 {
            self.direction_selector.motor.set_low();
            let value = if value == -128 { 127 } else { -value };
            self.motor
                .set_power(map_range(value as i32, 1, 127, 0, 255) as u8);
        } else {
            self.motor.set_power(value as u8);
        }
    }
    fn get_current_throttle(&self) -> i8 {
        let direction = self.direction_selector.motor.is_set_low();
        map_range(
            self.motor.get_power() as i32,
            0,
            255,
            if direction { -128 } else { 1 },
            if direction { -1 } else { 127 },
        ) as i8
    }
}

pub trait MotorSetup {
    fn moves(&mut self) -> bool;
    fn stop(&mut self);
}

pub struct RobotChassis<A, S> {
    pub accelerator: A,
    pub steering: S,
}

impl<A, S> RobotChassis<A, S>
where
    A: Accelerator,
    S: SteeringAxle,
{
    pub fn new(accelerator: A, steering: S) -> Self {
        Self {
            accelerator,
            steering,
        }
    }
}

impl<A> MotorSetup for RobotChassis<A, DummySteeringAxle>
where
    A: Accelerator,
{
    fn moves(&mut self) -> bool {
        self.accelerator.get_current_throttle() != 0
    }
    fn stop(&mut self) {
        self.accelerator.set_throttle(0);
    }
}

impl<A, M> MotorSetup for RobotChassis<A, BinarySteeringAxle<M>>
where
    A: Accelerator,
    M: MotorDriver,
{
    fn moves(&mut self) -> bool {
        self.accelerator.get_current_throttle() != 0 || self.steering.get_current_steering() != 0
    }
    fn stop(&mut self) {
        self.accelerator.set_throttle(0);
        self.steering.set_steering(0);
    }
}

impl<A, M> MotorSetup for RobotChassis<A, ServoSteeringAxle<M>>
where
    A: Accelerator,
    M: MotorDriver,
{
    fn moves(&mut self) -> bool {
        self.accelerator.get_current_throttle() != 0
    }
    fn stop(&mut self) {
        self.accelerator.set_throttle(0);
    }
}

pub struct Motors<A, S> {
    chassis: RobotChassis<A, S>,
}

impl<A, S> Motors<A, S> {
    pub fn setup(chassis: RobotChassis<A, S>) -> Self {
        Self { chassis }
    }
}

impl<A, S> Motors<A, S>
where
    A: Accelerator,
    S: SteeringAxle,
    RobotChassis<A, S>: MotorSetup,
{
    fn perform_action(
        &mut self,
        address: MotorAddress,
        value: u8,
    ) -> Result<MotorsStatus, PerformFunctionError> {
        let value: i8 = (value ^ 0x80) as i8;
        match address {
            MotorAddress::Stop => {
                self.chassis.stop();
                if self.chassis.moves() {
                    panic!("Rover moves while stopped!")
                }
                Ok(MotorsStatus::Steady)
            }
            MotorAddress::Accelerate => {
                self.chassis.accelerator.set_throttle(value);
                Ok(MotorsStatus::from(self.chassis.moves()))
            }
            MotorAddress::Steer => {
                self.chassis.steering.set_steering(value);

                Ok(MotorsStatus::from(self.chassis.moves()))
            }
            MotorAddress::UnknownAddress => Err(PerformFunctionError::IncorrectAddress),
        }
    }
}

pub enum MotorsStatus {
    Steady,
    Moving,
}

impl From<bool> for MotorsStatus {
    fn from(value: bool) -> Self {
        if value {
            MotorsStatus::Moving
        } else {
            MotorsStatus::Steady
        }
    }
}

pub enum MotorAddress {
    Stop,
    Accelerate,
    //Back,
    Steer,
    UnknownAddress,
}

impl From<u8> for MotorAddress {
    fn from(value: u8) -> Self {
        match value {
            0x0 => MotorAddress::Stop,
            0x1 => MotorAddress::Accelerate,
            0x2 => MotorAddress::UnknownAddress,
            0x3 => MotorAddress::Steer,
            _ => MotorAddress::UnknownAddress,
        }
    }
}

pub enum MotorFunctionCode {
    Dummy,
    Move,
    Config,
    UnknownAddress,
}

impl From<u8> for MotorFunctionCode {
    fn from(value: u8) -> Self {
        match value {
            0x0 => MotorFunctionCode::Dummy,
            0x1 => MotorFunctionCode::Move,
            0x2 => MotorFunctionCode::Config,
            _ => MotorFunctionCode::UnknownAddress,
        }
    }
}

impl<'a, A, S> AddressablePeripheral<'a, MotorsStatus /*, MotorAddress*/> for Motors<A, S>
where
    A: Accelerator,
    S: SteeringAxle,
    RobotChassis<A, S>: MotorSetup,
{
    fn perform_function(
        self: &mut Self,
        function_code: u8,
        address: u8,
        value: u8,
    ) -> Result<MotorsStatus, PerformFunctionError> {
        match MotorFunctionCode::from(function_code) {
            MotorFunctionCode::Dummy => {
                info!("Waiting for commands...");
                Ok(MotorsStatus::from(self.chassis.moves()))
            }
            MotorFunctionCode::Move => self.perform_action(MotorAddress::from(address), value),
            _ => Err(PerformFunctionError::WrongFunctionCode),
        }
    }
}
