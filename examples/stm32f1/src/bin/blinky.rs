#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_stm32::gpio::{Level, Output, Speed, Pin, AnyPin};
use embassy_time::Timer;
use embassy_stm32::time::Hertz;
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
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            // Oscillator for bluepill, Bypass for nucleos.
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }
    let p = embassy_stm32::init(config);
    //let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    spawner.spawn(blinky(p.PB0.degrade())).unwrap();
    let config = Config::default();
    let mut usart = Uart::new(p.USART2, p.PA3, p.PA2, Irqs, p.DMA1_CH7, p.DMA1_CH6, config).unwrap();
    //let mut led = Output::new(p.PB0, Level::High, Speed::Low);
    let mut rst = Output::new(p.PA0, Level::High, Speed::Low);
    Timer::after_millis(200).await;
    rst.set_low();
    Timer::after_millis(300).await;
    rst.set_high();
    Timer::after_millis(1500).await;

    let mut s  = [0u8; 1000];
    let mut t  = [0u8; 20];
    let mut a  = [0u8; 1];
    unwrap!(usart.write("+++".as_bytes()).await);
    //Timer::after_millis(100).await;
    info!("0\n");
    unwrap!(usart.read(&mut a).await);
    let str_resp = core::str::from_utf8(&a).unwrap();
    info!("1 {}\n", str_resp);
    unwrap!(usart.write("a".as_bytes()).await);
    Timer::after_millis(10).await;
    unwrap!(usart.write(b"at+h\r").await);
    unwrap!(usart.read(&mut s).await);
    let str_resp = core::str::from_utf8(&s).unwrap();
    info!("2 {}", str_resp);
    unwrap!(usart.read(&mut t).await);
    let str_resp = core::str::from_utf8(&t).unwrap();
    info!("3 {}", str_resp);
    loop {
     //   led.set_high();
        Timer::after_millis(1000).await;

     //   led.set_low();
        Timer::after_millis(1000).await;
    }
}
