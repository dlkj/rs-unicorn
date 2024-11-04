#![no_std]
#![no_main]

use bsp::entry;
use cortex_m::singleton;
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::{Alignment, Text},
};
use embedded_hal::digital::OutputPin;
use panic_probe as _;

use rp_pico::{self as bsp};

use bsp::hal::{
    clocks::init_clocks_and_plls, dma::DMAExt, pac, pio::PIOExt, sio::Sio, watchdog::Watchdog,
};

use rs_unicorn::{Unicorn, UnicornPins, HEIGHT};

/* Todo:
 *   - Check Pimoroni repos for updated pio code - https://github.com/pimoroni/pimoroni-pico/blob/main/libraries/galactic_unicorn/galactic_unicorn.pio
 *   - Measure and optimize rendering frame buffer to bit stream
 *     - byte per pixel vs nibble per pixel (See Galactic Unicorn code)
 *     - Only update changed pixels
 *     - Minimize read-modify-write operations
 */
#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let _clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led = pins.led.into_push_pull_output();

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    let unicorn_pins = UnicornPins {
        sin: pins.gpio8.into_function(),
        sclk: pins.gpio9.into_function(),
        latch: pins.gpio10.into_function(),
        blank: pins.gpio11.into_function(),
        sr0: pins.gpio22.into_function(),
        sr1: pins.gpio21.into_function(),
        sr2: pins.gpio20.into_function(),
        sr3: pins.gpio19.into_function(),
        sr4: pins.gpio18.into_function(),
        sr5: pins.gpio17.into_function(),
        sr6: pins.gpio16.into_function(),
    };

    let dma = pac.DMA.split(&mut pac.RESETS);

    let buf1 = singleton!(: [u32; 315] = [0; 315]).unwrap();
    let buf2 = singleton!(: [u32; 315] = [0; 315]).unwrap();

    let mut unicorn = Unicorn::new(&mut pio, sm0, unicorn_pins, dma.ch0, dma.ch1, buf1, buf2);

    unicorn.clear(Rgb888::CSS_MIDNIGHT_BLUE).unwrap();

    let text = "ABC";
    let character_style = MonoTextStyle::new(&FONT_5X8, Rgb888::CSS_DARK_GOLDENROD);
    Text::with_alignment(
        text,
        Point::new(1, HEIGHT as i32 - 1),
        character_style,
        Alignment::Left,
    )
    .draw(&mut unicorn)
    .unwrap();

    loop {
        led.set_high().unwrap();
        unicorn.flush();
        led.set_low().unwrap();
    }
}
