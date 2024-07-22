#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, AnyPin, Output, Pin, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_stm32::i2c::I2c;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
};

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        // 80Mhz clock (Source: 8 / SrcDiv: 1 * PllMul 20 / ClkDiv 2)
        // 80MHz highest frequency for flash 0 wait.
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(8),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL20,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2), // sysclk 80Mhz clock (8 / 1 * 20 / 2)
        });
        //config.rcc.hsi48 = Some(Default::default()); // needed for RNG
    }
    let p = embassy_stm32::init(config);

    spawner.spawn(blinky(p.PC13.degrade())).unwrap();
    let mut rst = Output::new(p.PE15, Level::Low, Speed::Low);
    Timer::after_millis(100).await;
    rst.set_high();
    Timer::after_millis(100).await;
    let mut i2c_cfg = embassy_stm32::i2c::Config::default();
    i2c_cfg.sda_pullup = true;
    i2c_cfg.scl_pullup = true;
    let i2c = I2c::new(
        p.I2C2,
        p.PB10,
        p.PB11,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH5,
        Hertz(400_000),
        i2c_cfg,
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);

    let im = Image::new(&raw, Point::new(32, 0));

    display.init().unwrap();
    im.draw(&mut display).unwrap();

    display.flush().unwrap();

    loop {
        Timer::after_millis(300).await;
    }
}
