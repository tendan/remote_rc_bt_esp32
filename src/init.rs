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

pub fn init_embassy_runtime(timg0: TIMG0<'static>) {
    let timer0 = TimerGroup::new(timg0);

    esp_preempt::start(timer0.timer0);

    esp_hal_embassy::init(timer0.timer1);

    info!("Embassy initialized!");
}
