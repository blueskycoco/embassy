#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_stm32::gpio::{Level, Output, Speed, Pin, AnyPin};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

#[embassy_executor::task]
async fn blinky(pin: AnyPin) {
    let mut led = Output::new(pin, Level::High, Speed::Low);

    loop {
        led.set_high();
        Timer::after_millis(300).await;

        led.set_low();
        Timer::after_millis(300).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    spawner.spawn(blinky(p.PB0.degrade())).unwrap();
    let config = Config::default();
    let mut usart = Uart::new(p.USART2, p.PA3, p.PA2, Irqs, p.DMA1_CH7, p.DMA1_CH6, config).unwrap();
    //let mut led = Output::new(p.PB0, Level::High, Speed::Low);
    let mut rst = Output::new(p.PA0, Level::High, Speed::Low);
    Timer::after_millis(200).await;
    rst.set_low();
    Timer::after_millis(100).await;
    rst.set_high();
    Timer::after_millis(100).await;

    let mut s  = [0u8; 16];
    let mut a  = [0u8; 1];
    unwrap!(usart.write(b"+++").await);
    unwrap!(usart.read(&mut a).await);
    info!("1 {}\n", a);
    unwrap!(usart.write(b"a").await);
    //Timer::after_millis(10).await;
    unwrap!(usart.write(b"at+e\r").await);
    unwrap!(usart.read(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("2 {}", str_resp);
    unwrap!(usart.read(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("{}", str_resp);
    unwrap!(usart.read(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("{}", str_resp);
    loop {
     //   led.set_high();
        Timer::after_millis(1000).await;

     //   led.set_low();
        Timer::after_millis(1000).await;
    }
}
