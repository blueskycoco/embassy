#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_stm32::exti::{ExtiInput, AnyChannel, Channel};
use embassy_stm32::gpio::{AnyPin, Level, Output, Pull, Pin, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use embedded_io::Write;

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

bind_interrupts!(struct Irqs2 {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

#[embassy_executor::task]
async fn blinky(pin: AnyPin) {
    let mut led = Output::new(pin, Level::High, Speed::Low);

    loop {
        //info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        //info!("low");
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

fn clear(ary: &mut [u8]) {
    ary.iter_mut().for_each(|m| *m = 0)
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");
    spawner.spawn(blinky(p.PA5.degrade())).unwrap();
    spawner.spawn(btn(p.PC13.degrade(), p.EXTI13.degrade())).unwrap();
    let config = Config::default();
    let mut usart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH2, p.DMA1_CH3, config).unwrap();
    let mut dbg = Uart::new(p.USART2, p.PA3, p.PA2, Irqs2, p.DMA1_CH4, p.DMA1_CH5, config).unwrap();
    let (mut tx, _rx) = dbg.split(); 
    let mut s  = [0u8; 64];

    /* switch from pass through to at command mode */
    unwrap!(usart.write("+++".as_bytes()).await);
    unwrap!(usart.read_until_idle(&mut s).await);
    unwrap!(usart.write("a".as_bytes()).await);
    unwrap!(usart.read_until_idle(&mut s).await);

    Timer::after_millis(100).await;
    let mut i: u32 = 0; 
    loop {
    unwrap!(usart.write("at+wann\r".as_bytes()).await);
    unwrap!(usart.read_until_idle(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("{}: {}", i, str_resp);
    writeln!(tx, "{}: {}\r\n", i, str_resp);
    i = i + 1;
    clear(&mut s);
    Timer::after_millis(2000).await;
    }
    /*unwrap!(usart.write("at+h\r".as_bytes()).await);
    for _ in 0..2 {
        unwrap!(usart.read_until_idle(&mut s).await);
        let str_resp = core::str::from_utf8(&s).unwrap();
        info!("{}", str_resp);
        clear(&mut s);
    }
    unwrap!(usart.write("at+wmode=apsta\r".as_bytes()).await);
    Timer::after_millis(10).await;
    unwrap!(usart.read_until_idle(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("{}", str_resp);
    clear(&mut s);
    unwrap!(usart.write("at+rptmac\r".as_bytes()).await);
    Timer::after_millis(10).await;
    unwrap!(usart.read_until_idle(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("{}", str_resp);
    
    unwrap!(usart.write("at+wskey=wpa2psk,aes,DUBB-JcJf-kU4g-C3IY\r".as_bytes()).await);
    Timer::after_millis(10).await;
    unwrap!(usart.read_until_idle(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("{}", str_resp);
    clear(&mut s);
    unwrap!(usart.write("at+wsssid=8848\r".as_bytes()).await);
    Timer::after_millis(10).await;
    unwrap!(usart.read_until_idle(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("{}", str_resp);
    clear(&mut s);
    unwrap!(usart.write("at+lann\r".as_bytes()).await);
    Timer::after_millis(10).await;
    unwrap!(usart.read_until_idle(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("{}", str_resp);
    clear(&mut s);
    unwrap!(usart.write("at+ping=172.20.10.3\r".as_bytes()).await);
    Timer::after_millis(10).await;
    unwrap!(usart.read_until_idle(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("{}", str_resp);
    clear(&mut s);*/
}
