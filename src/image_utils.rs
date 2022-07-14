use druid::{platform_menus::mac::file::print, Size};
use rand::Rng;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Point { x: x, y: y }
    }
}

pub type IPoint = Point<usize>;
pub type FPoint = Point<f64>;

impl IPoint {
    pub fn default() -> Self {
        IPoint { x: 0, y: 0 }
    }
}

impl From<FPoint> for IPoint {
    fn from(p: FPoint) -> Self {
        IPoint::new(p.x as usize, p.y as usize)
    }
}

impl FPoint {
    pub fn default() -> Self {
        FPoint { x: 0., y: 0. }
    }
}

impl From<IPoint> for FPoint {
    fn from(p: IPoint) -> Self {
        FPoint::new(p.x as f64, p.y as f64)
    }
}

#[derive(Clone, Debug, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGB {
    pub const BLACK: Self = RGB {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const TRANSPARENT: Self = RGB {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub fn rand() -> Self {
        let r: u8 = rand::thread_rng().gen();
        let g: u8 = rand::thread_rng().gen();
        let b: u8 = rand::thread_rng().gen();
        RGB {
            r: r,
            g: g,
            b: b,
            a: 255,
        }
    }

    pub fn interpolate(&self, other: &RGB, alpha: f64) -> Self {
        let a = alpha.clamp(0., 1.);
        RGB {
            r: (self.r as f64 * (1. - a) + other.r as f64 * a) as u8,
            g: (self.g as f64 * (1. - a) + other.g as f64 * a) as u8,
            b: (self.b as f64 * (1. - a) + other.b as f64 * a) as u8,
            a: (self.a as f64 * (1. - a) + other.a as f64 * a) as u8,
        }
    }

    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        let v = v.clamp(0., 1.);
        let hp = h / 60.0;
        let c = v * s;
        let x = c * (1. - ((hp % 2.) - 1.).abs());
        let mut rgb = [0., 0., 0.];

        if 0. <= hp && hp < 1. {
            rgb = [c, x, 0.];
        } else if 1. <= hp && hp < 2. {
            rgb = [x, c, 0.];
        } else if 2. <= hp && hp < 3. {
            rgb = [0., c, x];
        } else if 3. <= hp && hp < 4. {
            rgb = [0., x, c];
        } else if 4. <= hp && hp < 5. {
            rgb = [x, 0., c];
        } else if 5. <= hp && hp < 6. {
            rgb = [c, 0., x];
        }

        let m = v - c;
        rgb[0] += m;
        rgb[1] += m;
        rgb[2] += m;

        rgb[0] *= 255.;
        rgb[1] *= 255.;
        rgb[2] *= 255.;
        RGB {
            r: rgb[0] as u8,
            g: rgb[1] as u8,
            b: rgb[2] as u8,
            a: 255,
        }
    }

    pub fn resize_image(src: &[RGB], src_size: &IPoint, dst: &mut Vec<RGB>, dst_size: &IPoint) {
        assert!(src.len() == src_size.x * src_size.y);
        dst.resize(dst_size.x * dst_size.y, RGB::TRANSPARENT);
        let src_sizef: FPoint = (*src_size).into();
        let dst_sizef: FPoint = (*dst_size).into();
        // Resize ratio
        let mut r = src_sizef.x;
        let yd = -dst_sizef.y / 2. / dst_sizef.x;
        if yd * r + src_sizef.y / 2. < 0. {
            r = src_sizef.y * dst_sizef.x / dst_sizef.y;
        }

        for y in 0..dst_size.y {
            for x in 0..dst_size.x {
                let xd = (x as f64 - dst_sizef.x / 2.) / dst_sizef.x;
                let yd = (y as f64 - dst_sizef.y / 2.) / dst_sizef.x;
                let xs = (xd * r + src_sizef.x / 2.)
                    .floor()
                    .clamp(0., src_sizef.x - 1.);
                let ys = (yd * r + src_sizef.y / 2.)
                    .floor()
                    .clamp(0., src_sizef.y - 1.);
                let idxd = x + dst_size.x * y;
                let idxs = xs as usize + src_size.x * ys as usize;
                dst[idxd] = src[idxs];
            }
        }

        println!("sizes: ({:?},{:?})", src_size, dst_size);
    }

    pub fn create_image_data(src: &[RGB], dst: &mut Vec<u8>) {
        dst.resize(src.len() * 4, 0);
        for k in 0..src.len() {
            let rgb = src[k];
            dst[k * 4] = rgb.r;
            dst[k * 4 + 1] = rgb.g;
            dst[k * 4 + 2] = rgb.b;
            dst[k * 4 + 3] = rgb.a;
        }
    }
}
