// TODO: Implement those methods

use crate::control::instruction::AddressablePeripheral;
use crate::control::instruction::PerformFunctionError;
use esp_hal::gpio::Level;
use esp_hal::gpio::Output;
use log::info;

pub struct MotorSetup<'a> {
    pub(super) accelerator: Output<'a>,
    pub(super) backmove: Output<'a>,
    pub(super) steer_left: Output<'a>,
    pub(super) steer_right: Output<'a>,
}

pub struct Motors<'a> {
    motor_setup: MotorSetup<'a>,
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

impl<'a> AddressablePeripheral<'a, MotorsStatus /*, MotorAddress*/> for Motors<'a> {
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

impl<'a> Motors<'a> {
    pub fn setup(motor_setup: MotorSetup<'a>) -> Self {
        Self { motor_setup }
    }

    pub fn moves(self: &Self) -> bool {
        return self.motor_setup.accelerator.is_set_high()
            || self.motor_setup.backmove.is_set_high()
            || self.motor_setup.steer_left.is_set_high()
            || self.motor_setup.steer_right.is_set_high();
    }

    fn accelerate(self: &mut Self, enable: Level) {
        self.motor_setup.backmove.set_low();
        self.motor_setup.accelerator.set_level(enable);
    }

    fn backmove(self: &mut Self, enable: Level) {
        self.motor_setup.accelerator.set_low();
        self.motor_setup.backmove.set_level(enable);
    }

    fn go_left(self: &mut Self, enable: Level) {
        self.motor_setup.steer_right.set_low();
        self.motor_setup.steer_left.set_level(enable);
    }

    fn go_right(self: &mut Self, enable: Level) {
        self.motor_setup.steer_left.set_low();
        self.motor_setup.steer_right.set_level(enable);
    }

    pub fn stop(self: &mut Self) {
        self.motor_setup.accelerator.set_low();
        self.motor_setup.backmove.set_low();
        self.motor_setup.steer_left.set_low();
        self.motor_setup.steer_right.set_low();
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
