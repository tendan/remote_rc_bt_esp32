pub mod commands;
pub mod instruction;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Receiver;
use log::info;

use crate::control::commands::InstructionQueue;
use crate::control::instruction::{AddressablePeripheral, PerformFunctionError};
use crate::hardware::motor::Motors;

#[embassy_executor::task(pool_size = 1)]
pub async fn listen_to_commands(
    instruction_receiver: Receiver<'static, NoopRawMutex, [u8; 4], 4>,
    motors: &'static mut Motors<'static>,
) -> ! {
    loop {
        let [_, function_code, port_address, value] = instruction_receiver.receive().await;
        if let Err(code) = motors.perform_function(function_code, port_address, value) {
            match code {
                PerformFunctionError::WrongFunctionCode => {
                    info!(target: "steering_handle_task", "Wrong function code!")
                }
                PerformFunctionError::IncorrectAddress => {
                    info!(target: "steering_handle_task", "Incorrect address!")
                }
                PerformFunctionError::InvalidValue => {
                    info!(target: "steering_handle_task", "Invalid value!")
                }
            }
        }
    }
}
