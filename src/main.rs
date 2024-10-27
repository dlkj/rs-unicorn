#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use rp_pico::{self as bsp, hal::pio::PIOExt};

use bsp::hal::{clocks::init_clocks_and_plls, pac, sio::Sio, watchdog::Watchdog};

use rs_unicorn::{Unicorn, UnicornPins};

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

    let mut unicorn = Unicorn::new(&mut pio, sm0, unicorn_pins);

    loop {
        let colors = [(255, 0, 0), (0, 255, 0), (0, 0, 255)];
        let clear = (0, 0, 0);
        for color in colors {
            for y in 0..rs_unicorn::HEIGHT as u8 {
                for x in 0..rs_unicorn::WIDTH as u8 {
                    unicorn.set_pixel((x, y), color);
                }
            }
            unicorn.draw();
        }
        for y in 0..rs_unicorn::HEIGHT as u8 {
            for x in 0..rs_unicorn::WIDTH as u8 {
                unicorn.set_pixel((x, y), clear);
            }
        }
        unicorn.draw();
    }
}
