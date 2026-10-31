use num_complex::Complex64;
use std::f64::consts::{FRAC_1_SQRT_2, PI};

pub type Complex = Complex64;

pub const ZERO: Complex = Complex::new(0.0, 0.0);
pub const ONE: Complex = Complex::new(1.0, 0.0);
pub const I: Complex = Complex::new(0.0, 1.0);
pub const SQRT1_2: f64 = FRAC_1_SQRT_2;
pub const PI_CONST: f64 = PI;

/// Formats a complex number nicely, e.g. "0.7071", "0.7071 + 0.7071i", "-i", etc.
pub fn format_complex(c: Complex, precision: usize) -> String {
    let re = c.re;
    let im = c.im;
    let eps = 1e-10;

    let re_zero = re.abs() < eps;
    let im_zero = im.abs() < eps;

    if re_zero && im_zero {
        return format!("{:.prec$}", 0.0, prec = precision);
    }

    if im_zero {
        return format!("{:.prec$}", re, prec = precision);
    }

    if re_zero {
        if (im - 1.0).abs() < eps {
            return "i".to_string();
        } else if (im + 1.0).abs() < eps {
            return "-i".to_string();
        } else {
            return format!("{:.prec$}i", im, prec = precision);
        }
    }

    let sign = if im > 0.0 { "+" } else { "-" };
    let im_abs = im.abs();
    if (im_abs - 1.0).abs() < eps {
        format!("{:.prec$} {} i", re, sign, prec = precision)
    } else {
        format!("{:.prec$} {} {:.prec$}i", re, sign, im_abs, prec = precision)
    }
}
