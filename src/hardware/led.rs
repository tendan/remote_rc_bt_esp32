use embassy_time::Timer;
use esp_hal::gpio::Output;

pub async fn led_blink(mut time_ms: u64, period_ms: u64, led: &mut Output<'static>) {
    while time_ms > 0 {
        led.toggle();
        Timer::after_millis(period_ms).await;
        time_ms -= period_ms;
    }
}
