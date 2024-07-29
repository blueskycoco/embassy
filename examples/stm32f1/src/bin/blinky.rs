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

fn clear(ary: &mut [u8]) {
    ary.iter_mut().for_each(|m| *m = 0)
}

async fn usr_cmd(usart: &mut Uart<'_, embassy_stm32::mode::Async>, cmd: &str) {
    let mut s = [0u8; 128];
    unwrap!(usart.write(cmd.as_bytes()).await);
    loop {
        unwrap!(usart.read_until_idle(&mut s).await);
        let str_resp = core::str::from_utf8(&s).unwrap();
        info!("{}", str_resp);
        if str_resp.contains("+ok") || str_resp.contains("+ERR") {
            break;
        }
        clear(&mut s);
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
    let mut rst = Output::new(p.PA0, Level::High, Speed::Low);
    let mut s  = [0u8; 64];
    Timer::after_millis(200).await;
    rst.set_low();
    Timer::after_millis(300).await;
    rst.set_high();
    Timer::after_millis(1500).await;

    unwrap!(usart.write("+++".as_bytes()).await);
    unwrap!(usart.read_until_idle(&mut s).await);
    unwrap!(usart.write("a".as_bytes()).await);
    unwrap!(usart.read_until_idle(&mut s).await);

    Timer::after_millis(100).await;
    //usr_cmd(&mut usart, "at+wskey=wpa2psk,aes,DUBB-JcJf-kU4g-C3IY\r").await;
    //usr_cmd(&mut usart, "at+wsssid=8848\r").await;
    //usr_cmd(&mut usart, "at+wmode=sta\r").await;
    loop {
        usr_cmd(&mut usart, "at+wann\r").await;
        usr_cmd(&mut usart, "at+lann\r").await;
        usr_cmd(&mut usart, "at+ping=172.20.10.6\r").await;
        Timer::after_millis(2000).await;
    }
}
