use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};

pub type InstructionQueue = Channel<NoopRawMutex, [u8; 4], 4>;
pub type InstructionQueueReceiver<'d> = Receiver<'d, NoopRawMutex, [u8; 4], 4>;
pub type InstructionQueueSender<'d> = Sender<'d, NoopRawMutex, [u8; 4], 4>;
