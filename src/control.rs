use crate::control::commands::parse_command;

mod commands;
pub mod instruction;

#[embassy_executor::task(pool_size = 1)]
pub async fn listen_to_commands() -> ! {
    parse_command();
    loop {}
}
