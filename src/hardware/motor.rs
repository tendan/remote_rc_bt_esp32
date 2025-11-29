// TODO: Implement those methods

use crate::control::instruction::AddressablePeripheral;
use crate::control::instruction::PerformFunctionError;
use esp_hal::gpio::Level;
use esp_hal::gpio::Output;
use esp_hal::mcpwm::McPwm;
use esp_hal::mcpwm::PwmPeripheral;
use esp_hal::DriverMode;
use log::info;

pub trait MotorDriver {
    fn set_power(&mut self, value: u8);
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
}

// =============================
// TODO: DO NOT REMOVE THIS, IT MIGHT BE REALLY GOOD IDEA
pub struct BinaryMotor<'a> {
    pub motor: Output<'a>,
}
pub struct PwmMotor<'a, PWM> {
    pub motor: McPwm<'a, PWM>,
}

impl<'a> MotorDriver for BinaryMotor<'a> {
    fn set_power(&mut self, value: u8) {
        // TODO
    }
}

impl<'a, PWM: PwmPeripheral> MotorDriver for PwmMotor<'a, PWM> {
    fn set_power(&mut self, value: u8) {
        // TODO
    }
}
// =============================

// =============================
pub trait SteeringAxle {
    fn set_steering(&mut self, value: u8);
}

pub trait Accelerator {
    fn set_throttle(&mut self, value: u8);
}

pub struct BinarySteeringAxle<M>
where
    M: MotorDriver,
{
    pub motor_left: M,
    pub motor_right: M,
}

impl<M: MotorDriver> SteeringAxle for BinarySteeringAxle<M> {
    fn set_steering(&mut self, value: u8) {
        // TODO
    }
}

pub struct ServoSteeringAxle<M> {
    pub motor_steer: M,
}

impl<M: MotorDriver> SteeringAxle for ServoSteeringAxle<M> {
    fn set_steering(&mut self, value: u8) {
        // TODO
    }
}

pub struct BinaryAccelerator<MF, MB> {
    pub motor_forward: MF,
    pub motor_backward: MB,
}

impl<MF: MotorDriver, MB: MotorDriver> Accelerator for BinaryAccelerator<MF, MB> {
    fn set_throttle(&mut self, value: u8) {
        // TODO
    }
}

pub struct LinearAccelerator {}

impl Accelerator for LinearAccelerator {
    fn set_throttle(&mut self, value: u8) {
        // TODO
    }
}
// =============================

pub trait MotorSetup {
    fn moves(&mut self) -> bool;
    fn stop(&mut self) -> bool;
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
        true //TODO
    }
    fn stop(&mut self) -> bool {
        true //TODO
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
    fn stop(&mut self) -> bool {
        true //TODO
    }
}

pub struct Motors<C> {
    chassis: C,
}

impl<C> Motors<C> {
    pub fn new(chassis: C) -> Self {
        Self { chassis }
    }

    fn perform_action(
        self: &mut Self,
        address: MotorAddress,
        value: u8,
    ) -> Result<MotorsStatus, PerformFunctionError> {
        match address {
            MotorAddress::Stop => {
                self.stop();
                if self.moves() {
                    panic!("Rover moves while stopped!")
                }
                Ok(MotorsStatus::Steady)
            }
            MotorAddress::Accelerate => {
                if value > 0x1 {
                    return Err(PerformFunctionError::InvalidValue);
                }
                self.accelerate(Level::from(value == 0x1));
                Ok(MotorsStatus::from(self.moves()))
            }
            MotorAddress::Back => {
                if value > 0x1 {
                    return Err(PerformFunctionError::InvalidValue);
                }
                self.backmove(Level::from(value == 0x1));
                Ok(MotorsStatus::from(self.moves()))
            }
            MotorAddress::Steer => {
                match value {
                    0x0 => {
                        self.go_left(Level::Low);
                        self.go_right(Level::Low);
                        Ok(())
                    }
                    0x1 => {
                        self.go_left(Level::High);
                        Ok(())
                    }
                    0x2 => {
                        self.go_right(Level::High);
                        Ok(())
                    }
                    _ => Err(PerformFunctionError::InvalidValue),
                }?;

                Ok(MotorsStatus::from(self.moves()))
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
    Back,
    Steer,
    UnknownAddress,
}

impl From<u8> for MotorAddress {
    fn from(value: u8) -> Self {
        match value {
            0x0 => MotorAddress::Stop,
            0x1 => MotorAddress::Accelerate,
            0x2 => MotorAddress::Back,
            0x3 => MotorAddress::Steer,
            _ => MotorAddress::UnknownAddress,
        }
    }
}

impl<'a, A, S> AddressablePeripheral<'a, MotorsStatus /*, MotorAddress*/>
    for Motors<RobotChassis<A, S>>
where
    A: Accelerator,
    S: SteeringAxle,
{
    fn perform_function(
        self: &mut Self,
        function_code: u8,
        address: u8,
        value: u8,
    ) -> Result<MotorsStatus, PerformFunctionError> {
        match function_code {
            0x0 => {
                info!("Waiting for commands...");
                Ok(MotorsStatus::from(self.moves()))
            }
            0x1 => self.perform_action(MotorAddress::from(address), value),
            _ => Err(PerformFunctionError::WrongFunctionCode),
        }
    }
}
