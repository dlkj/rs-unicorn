#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::OutputPin;
use panic_probe as _;

use rp_pico::{self as bsp, hal::pio::PIOExt};

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use rs_unicorn::{Unicorn, UnicornPins};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
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

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

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

    let mut led_pin: rp_pico::hal::gpio::Pin<
        rp_pico::hal::gpio::bank0::Gpio25,
        rp_pico::hal::gpio::FunctionSio<rp_pico::hal::gpio::SioOutput>,
        rp_pico::hal::gpio::PullDown,
    > = pins.led.into_push_pull_output();

    loop {
        info!("on!");
        led_pin.set_high().unwrap();
        unicorn.set_on();
        delay.delay_ms(500);
        info!("off!");
        led_pin.set_low().unwrap();
        unicorn.set_off();
        delay.delay_ms(500);
    }
}
