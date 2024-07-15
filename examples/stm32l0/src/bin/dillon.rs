#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::{ExtiInput, AnyChannel, Channel};
use embassy_stm32::gpio::{AnyPin, Level, Output, Pull, Pin, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn blinky(pin: AnyPin) {
    let mut led = Output::new(pin, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}

#[embassy_executor::task]
async fn btn(pin: AnyPin, ch: AnyChannel) {
    let mut button = ExtiInput::new(pin, ch, Pull::Up);

    info!("Press the USER button...");

    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");
    spawner.spawn(blinky(p.PA5.degrade())).unwrap();
    spawner.spawn(btn(p.PC13.degrade(), p.EXTI13.degrade())).unwrap();
}
