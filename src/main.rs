#![no_std]
#![no_main]

use bsp::entry;
use cortex_m::singleton;
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{
    mono_font::{ascii::FONT_5X7, MonoTextStyle},
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
 *   - byte per pixel vs nibble per pixel
 *   - Check Pimoroni repos for updated pio code - https://github.com/pimoroni/pimoroni-pico/blob/main/libraries/galactic_unicorn/galactic_unicorn.pio
 *   - Separate frame buffer from bit stream
 *   - embedded graphics api support
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

    // `    let brightness = 100;
    //     let red = (brightness, 0, 0);
    //     let green = (0, brightness, 0);
    //     let blue = (0, 0, brightness);
    //     let white = (brightness, brightness, brightness);
    //     let black = (0, 0, 0);
    //     for y in 0..rs_unicorn::HEIGHT as u8 {
    //         let palette = if y % 4 < 2 {
    //             [white, red, green, blue, black, black, black, black]
    //         } else {
    //             [black, black, black, black, white, red, green, blue]
    //         };

    //         for x in 0..rs_unicorn::WIDTH as u8 {
    //             unicorn.draw()
    //             unicorn.set_pixel((x, y), palette[(x / 2) as usize]);
    //         }
    //     }`

    let character_style = MonoTextStyle::new(&FONT_5X7, Rgb888::CSS_DIM_GRAY);

    // Draw centered text.
    let text = "abc";
    Text::with_alignment(
        text,
        Point::new(0, HEIGHT as i32 - 1),
        character_style,
        Alignment::Left,
    )
    .draw(&mut unicorn)
    .unwrap();

    unicorn.flush();

    let text = "abc";
    Text::with_alignment(
        text,
        Point::new(0, HEIGHT as i32 - 1),
        character_style,
        Alignment::Left,
    )
    .draw(&mut unicorn)
    .unwrap();

    // let mut frame = 0;
    // let palette = [white, red, green, blue, black, black, black, black];
    loop {
        // for x in 0..rs_unicorn::WIDTH as u8 {
        //     unicorn.set_pixel(
        //         (x, 6),
        //         palette[((x + frame) % palette.len() as u8) as usize],
        //     );
        // }

        led.set_high().unwrap();
        unicorn.flush();
        led.set_low().unwrap();

        // frame += 1;
        // frame %= palette.len() as u8;
    }
}
