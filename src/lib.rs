#![no_std]

use rp_pico::{
    self as bsp,
    hal::pio::{self, PIOExt, PIO},
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

const ROW_COUNT2: usize = 7;
const ROW_BYTES: usize = 12;
const BCD_FRAMES: usize = 15; // includes fet discharge frame
const BIT_STREAM_LENGTH: usize = ROW_COUNT2 * ROW_BYTES * BCD_FRAMES;
pub const WIDTH: usize = 16;
pub const HEIGHT: usize = 7;

pub struct Unicorn<'a, P, SM>
where
    P: PIOExt,
    SM: pio::ValidStateMachine<PIO = P>,
{
    pio: &'a mut PIO<P>,
    sm: pio::StateMachine<SM, pio::Running>,
    tx: pio::Tx<SM>,
    pins: UnicornPins,
    bit_stream: [u8; BIT_STREAM_LENGTH],
}

impl<'a, P, SM> Unicorn<'a, P, (P, SM)>
where
    P: PIOExt,
    SM: pio::StateMachineIndex,
{
    pub fn new(
        pio: &'a mut PIO<P>,
        sm: pio::UninitStateMachine<(P, SM)>,
        pins: UnicornPins,
    ) -> Unicorn<'a, P, (P, SM)> {
        let pio_program = Self::assemble_pio_program();
        let installed = pio.install(&pio_program).unwrap();

        let (mut sm, _rx, tx) = pio::PIOBuilder::from_installed_program(installed)
            .buffers(bsp::hal::pio::Buffers::OnlyTx)
            .out_pins(pins.sr6.id().num, 7)
            .side_set_pin_base(pins.sclk.id().num)
            .set_pins(pins.sin.id().num, 4)
            .autopull(true)
            .pull_threshold(32)
            .out_shift_direction(pio::ShiftDirection::Right)
            .clock_divisor_fixed_point(1, 0)
            .build(sm);

        sm.set_pins([
            (pins.blank.id().num, pio::PinState::High),
            (pins.latch.id().num, pio::PinState::High),
            (pins.sclk.id().num, pio::PinState::High),
            (pins.sin.id().num, pio::PinState::High),
            (pins.sr0.id().num, pio::PinState::High),
            (pins.sr1.id().num, pio::PinState::High),
            (pins.sr2.id().num, pio::PinState::High),
            (pins.sr3.id().num, pio::PinState::High),
            (pins.sr4.id().num, pio::PinState::High),
            (pins.sr5.id().num, pio::PinState::High),
            (pins.sr6.id().num, pio::PinState::High),
        ]);

        sm.set_pindirs([
            (pins.blank.id().num, pio::PinDir::Output),
            (pins.latch.id().num, pio::PinDir::Output),
            (pins.sclk.id().num, pio::PinDir::Output),
            (pins.sin.id().num, pio::PinDir::Output),
            (pins.sr0.id().num, pio::PinDir::Output),
            (pins.sr1.id().num, pio::PinDir::Output),
            (pins.sr2.id().num, pio::PinDir::Output),
            (pins.sr3.id().num, pio::PinDir::Output),
            (pins.sr4.id().num, pio::PinDir::Output),
            (pins.sr5.id().num, pio::PinDir::Output),
            (pins.sr6.id().num, pio::PinDir::Output),
        ]);

        let sm = sm.start();

        Self {
            pio,
            sm,
            tx,
            pins,
            bit_stream: Self::init_bit_stream(),
        }
    }

    fn init_bit_stream() -> [u8; BIT_STREAM_LENGTH] {
        let mut bit_stream = [0; BIT_STREAM_LENGTH];

        // initialize the bcd timing values and row selects in the bit stream
        for row in 0..HEIGHT {
            for frame in 0..BCD_FRAMES {
                // determine offset in the buffer for this row/frame
                let offset = (row * ROW_BYTES * BCD_FRAMES) + (ROW_BYTES * frame);

                let row_select_offset = offset + 9;
                let bcd_offset = offset + 10;

                // the last bcd frame is used to allow the fets to discharge to avoid ghosting
                if frame == BCD_FRAMES - 1usize {
                    let bcd_ticks: u16 = 65535;
                    bit_stream[row_select_offset] = 0b11111111;
                    bit_stream[bcd_offset + 1] = ((bcd_ticks & 0xff00) >> 8) as u8;
                    bit_stream[bcd_offset] = (bcd_ticks & 0xff) as u8;
                    for col in 0..6 {
                        bit_stream[offset + col] = 0xff;
                    }
                } else {
                    let row_select_mask = !(1 << (7 - row));
                    let bcd_ticks: u16 = 1 << frame;
                    bit_stream[row_select_offset] = row_select_mask;
                    bit_stream[bcd_offset + 1] = ((bcd_ticks & 0xff00) >> 8) as u8;
                    bit_stream[bcd_offset] = (bcd_ticks & 0xff) as u8;
                }
            }
        }

        bit_stream
    }

    fn assemble_pio_program() -> ::pio::Program<32_usize> {
        pio_proc::pio_asm!(
            ".side_set 1 opt"

                ".wrap_target"
                    // clock out 16 pixels worth of data
                    "set y, 15"                // 15 because `jmp` test is pre decrement

                    "pixels:"

                    "pull ifempty"

                    // dummy bit used to align pixel data to nibbles
                    "out null, 1  "            // discard

                    // red bit
                    "out x, 1       side 0 "   // pull in first bit from OSR into register X, clear clock
                    "set pins, 8"              // clear data bit (maintain blank)
                    "jmp !x endr"              // if bit was zero jump to endr
                    "set pins, 9"              // set data bit (maintain blank)
                "endr: "                       //
                    "nop            side 1"    // clock in bit

                    // green bit
                    "out x, 1       side 0"    // pull in first bit from OSR into register X, clear clock
                    "set pins, 8"              // clear data bit (maintain blank)
                    "jmp !x endg"              // if bit was zero jump to endg
                    "set pins, 9"              // set data bit (maintain blank)
                "endg:"                        //
                    "nop            side 1"    // clock in bit

                    // blue bit
                    "out x, 1       side 0"    // pull in first bit from OSR into register X, clear clock
                    "set pins, 8"              // clear data bit (maintain blank)
                    "jmp !x endb"              // if bit was zero jump to endb
                    "set pins, 9"              // set data bit (maintain blank)
                "endb:"                        //
                    "nop            side 1"    // clock in bit

                    "jmp y-- pixels"           // jump back to start of pixel loop

                    "pull"

                    // dummy byte to 32 bit align row data
                    "out null, 8"

                    // select active row
                    "out null, 1"              // discard dummy bit
                    "out pins, 7"              // output row selection mask

                    // pull bcd tick count into x register
                    "out x, 16"

                    // set latch pin to output column data on shift registers
                    "set pins, 12"             // set latch pin (while keeping blank high)

                    // set blank pin to enable column drivers
                    "set pins, 4"

                "bcd_count:"
                    "jmp x-- bcd_count "      // loop until bcd delay complete

                    // disable all row outputs
                    "set x, 0"                // load x register with 0 (we can't set more than 5 bits at a time)
                    "mov pins, !x"            // write inverted x (0xff) to row pins latching them all high

                    // disable led output (blank) and clear latch pin
                    "set pins, 8"

                ".wrap"
        )
        .program
    }
}

static GAMMA_14BIT: [u16; 256] = [
    0, 0, 0, 1, 2, 3, 4, 6, 8, 10, 13, 16, 20, 23, 28, 32, 37, 42, 48, 54, 61, 67, 75, 82, 90, 99,
    108, 117, 127, 137, 148, 159, 170, 182, 195, 207, 221, 234, 249, 263, 278, 294, 310, 326, 343,
    361, 379, 397, 416, 435, 455, 475, 496, 517, 539, 561, 583, 607, 630, 654, 679, 704, 730, 756,
    783, 810, 838, 866, 894, 924, 953, 983, 1014, 1045, 1077, 1110, 1142, 1176, 1210, 1244, 1279,
    1314, 1350, 1387, 1424, 1461, 1499, 1538, 1577, 1617, 1657, 1698, 1739, 1781, 1823, 1866, 1910,
    1954, 1998, 2044, 2089, 2136, 2182, 2230, 2278, 2326, 2375, 2425, 2475, 2525, 2577, 2629, 2681,
    2734, 2787, 2841, 2896, 2951, 3007, 3063, 3120, 3178, 3236, 3295, 3354, 3414, 3474, 3535, 3596,
    3658, 3721, 3784, 3848, 3913, 3978, 4043, 4110, 4176, 4244, 4312, 4380, 4449, 4519, 4589, 4660,
    4732, 4804, 4876, 4950, 5024, 5098, 5173, 5249, 5325, 5402, 5479, 5557, 5636, 5715, 5795, 5876,
    5957, 6039, 6121, 6204, 6287, 6372, 6456, 6542, 6628, 6714, 6801, 6889, 6978, 7067, 7156, 7247,
    7337, 7429, 7521, 7614, 7707, 7801, 7896, 7991, 8087, 8183, 8281, 8378, 8477, 8576, 8675, 8775,
    8876, 8978, 9080, 9183, 9286, 9390, 9495, 9600, 9706, 9812, 9920, 10027, 10136, 10245, 10355,
    10465, 10576, 10688, 10800, 10913, 11027, 11141, 11256, 11371, 11487, 11604, 11721, 11840,
    11958, 12078, 12198, 12318, 12440, 12562, 12684, 12807, 12931, 13056, 13181, 13307, 13433,
    13561, 13688, 13817, 13946, 14076, 14206, 14337, 14469, 14602, 14735, 14868, 15003, 15138,
    15273, 15410, 15547, 15685, 15823, 15962, 16102, 16242, 16383,
];

impl<'pio, P, SM> Unicorn<'pio, P, SM>
where
    P: PIOExt,
    SM: pio::ValidStateMachine<PIO = P>,
{
    #[inline(always)]
    pub fn set_pixel(&mut self, (x, y): (u8, u8), (r, g, b): (u8, u8, u8)) {
        self.set_pixel_rgb(x, y, r, g, b)
    }

    pub fn set_pixel_rgb(&mut self, x: u8, y: u8, r: u8, g: u8, b: u8) {
        let x = x as usize;
        let y = y as usize;
        if x >= WIDTH || y >= HEIGHT {
            return;
        }

        // make those coordinates sane
        let x = (WIDTH - 1) - x;

        // work out the byte offset of this pixel
        let byte_offset = x / 2;

        // check if it's the high or low nibble and create mask and shift value
        let shift = if x % 2 == 0 { 0 } else { 4 };
        let nibble_mask = 0b00001111 << shift;

        let mut gr = GAMMA_14BIT[r as usize];
        let mut gg = GAMMA_14BIT[g as usize];
        let mut gb = GAMMA_14BIT[b as usize];

        // set the appropriate bits in the separate bcd frames
        for frame in 0..BCD_FRAMES {
            // determine offset in the buffer for this row/frame
            let offset = (y * ROW_BYTES * BCD_FRAMES) + (ROW_BYTES * frame);

            let mut rgbd = ((gr & 0b1) << 1) | ((gg & 0b1) << 3) | ((gb & 0b1) << 2);

            // shift to correct nibble
            rgbd <<= shift;

            // clear existing data
            self.bit_stream[offset + byte_offset] &= !nibble_mask;

            // set new data
            self.bit_stream[offset + byte_offset] |= rgbd as u8;

            gr >>= 1;
            gg >>= 1;
            gb >>= 1;
        }
    }

    pub fn draw(&mut self) {
        for batch in self.bit_stream.chunks_exact(4) {
            while !self.tx.write(u32::from_le_bytes(unsafe {
                batch.try_into().unwrap_unchecked()
            })) {}
        }
    }
}
