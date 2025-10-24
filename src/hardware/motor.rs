// TODO: Implement those methods

use esp_hal::gpio::Output;

pub struct MotorSetup<'a> {
    pub(super) accelerator: Output<'a>,
    pub(super) backmove: Output<'a>,
    pub(super) steer_left: Output<'a>,
    pub(super) steer_right: Output<'a>
}

pub struct Motors<'a> {
    motor_setup: MotorSetup<'a>,
}

impl<'a> Motors<'a> {
    pub fn setup(motor_setup: MotorSetup<'a>) -> Self {
        Self {
            motor_setup
        }
    }
    // TODO: Implement methods related with steering
}

#[warn(dead_code)]
pub fn accelerate() {}

#[warn(dead_code)]
pub fn brake() {}
