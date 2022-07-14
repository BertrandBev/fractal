use druid::{kurbo::Circle, Size};

use crate::image_utils::{FPoint, IPoint, RGB};

pub struct Complex {
    pub r: f64,
    pub i: f64,
}

impl Complex {
    pub fn zero() -> Self {
        Complex { r: 0., i: 0. }
    }
}

pub struct ConvResult {
    max_iter: usize,
    iter: usize,
    norm_sqr: f64,
}

pub fn color_scheme(res: &ConvResult) -> RGB {
    if res.iter == res.max_iter {
        return RGB::BLACK;
    }
    let l = 1. / 2_f64.log2();
    let v = 5. + res.iter as f64 - (0.5_f64.log2() - res.norm_sqr.log2().log2()) * l;
    let mut res = RGB::from_hsv(
        360. * v / res.max_iter as f64,
        1.,
        10. * v / res.max_iter as f64,
    );
    let b = res.b;
    res.b = res.r;
    res.r = b;
    res
}

pub fn px_to_world(focus: &Circle, px_size: &Size, point: &IPoint) -> FPoint {
    //
    let mut xr = point.x as f64 / px_size.width;
    let mut yr = point.y as f64 / px_size.height;
    // Now transform back to window
    xr = (xr * 2. - 1.) * focus.radius;
    yr = (yr * 2. - 1.) * focus.radius;
    if px_size.width > px_size.height {
        xr *= px_size.width / px_size.height;
    } else {
        yr *= px_size.height / px_size.width;
    }
    FPoint {
        x: xr + focus.center.x,
        y: yr + focus.center.y,
    }
}

pub fn mandelbrot(c: Complex, escape_radius_sqr: f64, max_iter: usize) -> ConvResult {
    let mut z = Complex::zero();
    let mut z_sqr = Complex::zero();
    let mut iter = 0;
    let mut terminate_iter = 0;
    const EXTRA_ITER: usize = 5;

    loop {
        z.i = 2. * z.r * z.i + c.i;
        z.r = z_sqr.r - z_sqr.i + c.r;
        z_sqr.r = z.r * z.r;
        z_sqr.i = z.i * z.i;

        if terminate_iter == 0 {
            iter += 1;
            if iter >= max_iter || z_sqr.r + z_sqr.i > escape_radius_sqr {
                terminate_iter = 1;
            }
        } else {
            terminate_iter += 1;
            if terminate_iter > EXTRA_ITER {
                break;
            }
        }
    }

    ConvResult {
        max_iter: max_iter,
        iter: iter,
        norm_sqr: z_sqr.i + z_sqr.r,
    }
}
