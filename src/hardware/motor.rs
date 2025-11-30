// TODO: Implement those methods

use crate::control::instruction::AddressablePeripheral;
use crate::control::instruction::PerformFunctionError;
use esp_hal::gpio::Level;
use esp_hal::gpio::Output;
use esp_hal::mcpwm::McPwm;
use esp_hal::mcpwm::PwmPeripheral;
use log::info;

pub trait MotorDriver {
    fn set_power(&mut self, value: i8);
    fn get_power(&self) -> i8;
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
    fn set_power(&mut self, power: i8) {
        match self {
            Self::Binary(b) => b.set_power(power),
            Self::Pwm(p) => p.set_power(power),
        }
    }
    fn get_power(&self) -> i8 {
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
pub struct PwmMotor<'a, PWM> {
    pub motor: McPwm<'a, PWM>,
}

impl<'a> MotorDriver for BinaryMotor<'a> {
    fn set_power(&mut self, value: i8) {
        self.motor.set_level(Level::from(value != 0));
    }
    fn get_power(&self) -> i8 {
        match self.motor.output_level() {
            Level::Low => 0,
            Level::High => 1,
        }
    }
}

impl<'a, PWM: PwmPeripheral> MotorDriver for PwmMotor<'a, PWM> {
    fn set_power(&mut self, value: i8) {
        // TODO
    }
    fn get_power(&self) -> i8 {
        // TODO
        0
    }
}
// =============================

// =============================
pub trait SteeringAxle {
    fn set_steering(&mut self, value: i8);
    fn get_current_steering(&self) -> i8;
}

pub trait Accelerator {
    fn set_throttle(&mut self, value: i8);
    fn get_current_throttle(&self) -> i8;
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
            self.motor_right.set_power(1);
            self.motor_left.set_power(0);
        } else if value < 0 {
            self.motor_left.set_power(1);
            self.motor_right.set_power(0);
        } else {
            self.motor_left.set_power(0);
            self.motor_right.set_power(0);
        }
    }
    fn get_current_steering(&self) -> i8 {
        if self.motor_right.get_power() == 1 {
            1
        } else if self.motor_left.get_power() == 1 {
            -1
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
        // TODO
    }
    fn get_current_steering(&self) -> i8 {
        // TODO
        0
    }
}

pub struct BinaryAccelerator<MF, MB> {
    pub motor_forward: MF,
    pub motor_backward: MB,
}

impl<MF: MotorDriver, MB: MotorDriver> Accelerator for BinaryAccelerator<MF, MB> {
    fn set_throttle(&mut self, value: i8) {
        if value > 0 {
            self.motor_forward.set_power(1);
            self.motor_backward.set_power(0);
        } else if value < 0 {
            self.motor_backward.set_power(1);
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

pub struct LinearAccelerator {}

impl Accelerator for LinearAccelerator {
    fn set_throttle(&mut self, value: i8) {
        // TODO
    }
    fn get_current_throttle(&self) -> i8 {
        // TODO
        0
    }
}
// =============================

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
        true //TODO
    }
    fn stop(&mut self) {
        //TODO
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
                if value > 0x1 {
                    return Err(PerformFunctionError::InvalidValue);
                }
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
