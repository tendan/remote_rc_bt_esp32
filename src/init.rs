use esp_hal::clock::CpuClock;
use esp_hal::peripherals::{Peripherals, TIMG0};
use esp_hal::timer::timg::TimerGroup;
use esp_radio::Controller;
use log::info;
use static_cell::StaticCell;

pub fn init_core_system() -> Peripherals {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    peripherals
}
