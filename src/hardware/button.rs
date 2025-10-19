use embassy_futures::select::select;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Input;

pub async fn button_pressed_for(millis: Duration, button: &mut Input<'static>) -> bool {
    let button_released_too_early = button.wait_for_high();
    let button_held_for_proper_amount = Timer::after(millis);

    select(button_held_for_proper_amount, button_released_too_early)
        .await
        .is_first()
}
