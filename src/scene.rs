use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{DrawTarget, Point},
    Drawable, Pixel,
};
use fugit::MicrosDurationU64;

use crate::{HEIGHT, WIDTH};

pub trait Scene {
    fn tick<D>(&mut self, count: D)
    where
        D: Into<MicrosDurationU64>;
    fn draw<T>(&self, draw_target: &mut T)
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

    fn draw<T>(&self, draw_target: &mut T)
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
