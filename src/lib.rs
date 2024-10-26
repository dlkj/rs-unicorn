#![no_std]

use rp_pico::{
    self as bsp,
    hal::pio::{
        PIOExt, Running, StateMachine, StateMachineIndex, Tx, UninitStateMachine,
        ValidStateMachine, PIO,
    },
};

use bsp::hal::gpio::{bank0::*, FunctionPio0, Pin, PullDown};

pub struct UnicornPins {
    pub sin: Pin<Gpio8, FunctionPio0, PullDown>,
    pub sclk: Pin<Gpio9, FunctionPio0, PullDown>,
    pub latch: Pin<Gpio10, FunctionPio0, PullDown>,
    pub blank: Pin<Gpio11, FunctionPio0, PullDown>,
    pub sr0: Pin<Gpio22, FunctionPio0, PullDown>,
    pub sr1: Pin<Gpio21, FunctionPio0, PullDown>,
    pub sr2: Pin<Gpio20, FunctionPio0, PullDown>,
    pub sr3: Pin<Gpio19, FunctionPio0, PullDown>,
    pub sr4: Pin<Gpio18, FunctionPio0, PullDown>,
    pub sr5: Pin<Gpio17, FunctionPio0, PullDown>,
    pub sr6: Pin<Gpio16, FunctionPio0, PullDown>,
}

pub struct Unicorn<'a, P, SM>
where
    P: PIOExt,
    SM: ValidStateMachine<PIO = P>,
{
    pio: &'a mut PIO<P>,
    sm: StateMachine<SM, Running>,
    tx: Tx<SM>,
    pins: UnicornPins,
}

impl<'a, P, SM> Unicorn<'a, P, (P, SM)>
where
    P: PIOExt,
    SM: StateMachineIndex,
{
    pub fn new(
        pio: &'a mut PIO<P>,
        sm: UninitStateMachine<(P, SM)>,
        pins: UnicornPins,
    ) -> Unicorn<'a, P, (P, SM)> {
        todo!()
    }

    pub fn set_on(&mut self) {}
    pub fn set_off(&mut self) {}
}
