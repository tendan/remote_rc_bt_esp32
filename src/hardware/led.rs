use embassy_time::{Duration, Timer};
use esp_hal::gpio::Output;

pub async fn led_blink(mut time: Duration, period: Duration, led: &mut Output<'static>) {
    while time.as_millis() != 0 {
        led.toggle();
        Timer::after(period).await;
        time -= period;
    }
}
