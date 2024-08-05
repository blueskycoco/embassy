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
use crate::usart::Error;

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

async fn usr_cmd(usart: &mut Uart<'_, embassy_stm32::mode::Async>, tx: &mut dyn Write<Error = Error>, cmd: &str) {
    let mut s = [0u8; 128];
    unwrap!(usart.write(cmd.as_bytes()).await);
    loop {
        unwrap!(usart.read_until_idle(&mut s).await);
        let str_resp = core::str::from_utf8(&s).unwrap();
        info!("{}", str_resp);
        writeln!(tx, "{}\r\n", str_resp).unwrap();
        if str_resp.contains("+ok") || str_resp.contains("+ERR") {
            break;
        }
        clear(&mut s);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");
    spawner.spawn(blinky(p.PA5.degrade())).unwrap();
    spawner.spawn(btn(p.PC13.degrade(), p.EXTI13.degrade())).unwrap();
    let config = Config::default();
    let mut usart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH2, p.DMA1_CH3, config).unwrap();
    let dbg = Uart::new(p.USART2, p.PA3, p.PA2, Irqs2, p.DMA1_CH4, p.DMA1_CH5, config).unwrap();
    let (mut tx, _rx) = dbg.split(); 
    let mut s  = [0u8; 64];

    /* switch from pass through to at command mode */
    unwrap!(usart.write("+++".as_bytes()).await);
    unwrap!(usart.read_until_idle(&mut s).await);
    unwrap!(usart.write("a".as_bytes()).await);
    unwrap!(usart.read_until_idle(&mut s).await);

    Timer::after_millis(200).await;
    usr_cmd(&mut usart, &mut tx, "at+wmode=apsta\r").await;
    usr_cmd(&mut usart, &mut tx, "at+netp=TCP,Server,1234,172.20.10.2\r").await;
    usr_cmd(&mut usart, &mut tx, "at+tcpdis=on\r").await;
    loop {
        usr_cmd(&mut usart, &mut tx, "at+wann\r").await;
        usr_cmd(&mut usart, &mut tx, "at+netp\r").await;
        usr_cmd(&mut usart, &mut tx, "at+tcplk\r").await;
        usr_cmd(&mut usart, &mut tx, "at+ping=172.20.10.6\r").await;
        Timer::after_millis(2000).await;
    }
}
