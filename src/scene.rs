use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{DrawTarget, Point, RgbColor},
    Drawable, Pixel,
};
use fugit::ExtU64;
use fugit::MicrosDurationU64;

use crate::{HEIGHT, WIDTH};

use rand::{rngs::SmallRng, seq::SliceRandom};
use rand::{Rng, SeedableRng};

pub trait Scene {
    fn tick<D>(&mut self, count: D)
    where
        D: Into<MicrosDurationU64>;
    fn draw<T>(&mut self, draw_target: &mut T)
    where
        T: DrawTarget<Color = Rgb888, Error = ()>;
}

#[derive(Default)]
pub struct ColorWheel {
    offset: u8,
}

impl ColorWheel {
    fn wheel(mut wheel_pos: u8) -> Rgb888 {
        wheel_pos = 255 - wheel_pos;
        if wheel_pos < 85 {
            return Rgb888::new(255 - wheel_pos * 3, 0, wheel_pos * 3);
        }
        if wheel_pos < 170 {
            wheel_pos -= 85;
            return Rgb888::new(0, wheel_pos * 3, 255 - wheel_pos * 3);
        }
        wheel_pos -= 170;
        Rgb888::new(wheel_pos * 3, 255 - wheel_pos * 3, 0)
    }
}

impl Scene for ColorWheel {
    fn tick<D>(&mut self, count: D)
    where
        D: Into<MicrosDurationU64>,
    {
        self.offset = self
            .offset
            .wrapping_add(((count.into().ticks() / (1024 * 8)) & 0xff) as u8)
    }

    fn draw<T>(&mut self, draw_target: &mut T)
    where
        T: DrawTarget<Color = Rgb888, Error = ()>,
    {
        for x in 0..WIDTH {
            let color =
                Self::wheel((((x * 256) as u16 / WIDTH as u16 + self.offset as u16) & 255) as u8);

            for y in 0..HEIGHT {
                Pixel(Point::new(x as i32, y as i32), color)
                    .draw(draw_target)
                    .unwrap()
            }
        }
    }
}

pub struct DiscoFloor {
    last_update: MicrosDurationU64,
    pallet: [Rgb888; 4],
    rng: SmallRng,
}

impl DiscoFloor {}

impl Default for DiscoFloor {
    fn default() -> Self {
        Self {
            last_update: MicrosDurationU64::from_ticks(0),
            pallet: [Rgb888::RED, Rgb888::GREEN, Rgb888::BLUE, Rgb888::YELLOW],
            rng: SmallRng::seed_from_u64(0),
        }
    }
}

impl Scene for DiscoFloor {
    fn tick<D>(&mut self, count: D)
    where
        D: Into<MicrosDurationU64>,
    {
        self.last_update += count.into();
        if self.last_update > 1.secs::<1, 1_000_000>() {
            self.pallet.shuffle(&mut self.rng);
            self.last_update = 0.secs();
        }
    }

    fn draw<T>(&mut self, draw_target: &mut T)
    where
        T: DrawTarget<Color = Rgb888, Error = ()>,
    {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let color = self.pallet[((x / 2) % 2) + ((y / 2) % 2) * 2];

                Pixel(Point::new(x as i32, y as i32), color)
                    .draw(draw_target)
                    .unwrap()
            }
        }
    }
}

pub struct Sparkle {
    rng: SmallRng,
}

impl Sparkle {}

impl Default for Sparkle {
    fn default() -> Self {
        Self {
            rng: SmallRng::seed_from_u64(0),
        }
    }
}

impl Scene for Sparkle {
    fn tick<D>(&mut self, count: D)
    where
        D: Into<MicrosDurationU64>,
    {
    }

    fn draw<T>(&mut self, draw_target: &mut T)
    where
        T: DrawTarget<Color = Rgb888, Error = ()>,
    {
        draw_target.clear(Rgb888::BLACK).unwrap();

        for _ in 0..WIDTH {
            Pixel(
                Point::new(
                    self.rng.gen_range(0..WIDTH as i32),
                    self.rng.gen_range(0..HEIGHT as i32),
                ),
                Rgb888::new(
                    self.rng.gen_range(0..255),
                    self.rng.gen_range(0..255),
                    self.rng.gen_range(0..255),
                ),
            )
            .draw(draw_target)
            .unwrap()
        }
    }
}
