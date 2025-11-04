pub mod commands;
pub mod instruction;

use embassy_time::Timer;
use log::info;

use crate::control::commands::InstructionQueueReceiver;
use crate::control::instruction::{AddressablePeripheral, PerformFunctionError};
use crate::hardware::motor::Motors;

#[embassy_executor::task(pool_size = 1)]
pub async fn listen_to_commands(
    instruction_receiver: InstructionQueueReceiver<'static>,
    mut motors: Motors<'static>,
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
        Timer::after_millis(20).await;
    }
}
