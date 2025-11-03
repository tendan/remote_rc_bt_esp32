use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex, RawMutex};
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};

type InstructionArray = [u8; 4];
const INSTRUCTION_FIFO_SIZE: usize = 4;
pub type InstructionQueue =
    Channel<CriticalSectionRawMutex, InstructionArray, INSTRUCTION_FIFO_SIZE>;
pub type InstructionQueueReceiver<'d> =
    Receiver<'d, CriticalSectionRawMutex, InstructionArray, INSTRUCTION_FIFO_SIZE>;
pub type InstructionQueueSender<'d> =
    Sender<'d, CriticalSectionRawMutex, InstructionArray, INSTRUCTION_FIFO_SIZE>;
