extern crate image;

use image::ColorType;
use std::fs::File;
use image::png::PNGEncoder;

#[derive(Debug, Clone)]
struct PolarVector {
    pub r: f64,
    pub phi: f64,
    pub theta: f64,
}

#[derive(Debug, Clone)]
struct EuclidVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

fn coords_to_euclid_vector(
    (x, y, z): (usize, usize, usize),
    (angle_xy, angle_xz, angle_yz): (f64, f64, f64),
    side: usize,
    bailout: f64,
) -> EuclidVector {
    let half_side = (side / 2) as f64;
    let (mut xf, mut yf, mut zf) = (
        (x as f64 - half_side) / half_side * bailout,
        (y as f64 - half_side) / half_side * bailout,
        (z as f64 - half_side) / half_side * bailout
    );
    let (a, b) = (
        xf * angle_xy.cos() - yf * angle_xy.sin(),
        xf * angle_xy.sin() + yf * angle_xy.cos()
    );
    xf = a;
    yf = b;
    let (a, b) = (
        xf * angle_xz.cos() - zf * angle_xz.sin(),
        xf * angle_xz.sin() + zf * angle_xz.cos()
    );
    xf = a;
    zf = b;
    let (a, b) = (
        yf * angle_yz.cos() - zf * angle_yz.sin(),
        yf * angle_yz.sin() + zf * angle_yz.cos()
    );
    yf = a;
    zf = b;
    EuclidVector { x: xf, y: yf, z: zf }
}

fn euclid_vector_to_polar(vector: &EuclidVector) -> PolarVector {
    let (x, y, z) = (vector.x, vector.y, vector.z);
    let r = (x * x + y * y + z * z).sqrt();
    let phi = match x == 0.0 {
        true => 0.0,
        false => (y / x).atan()
    };
    let theta = match r == 0.0 {
        true => 0.0,
        false => (z / r).acos()
    };
    PolarVector { r, phi, theta }
}

fn iterate_euclid_vector(
    vector: &mut EuclidVector,
    vector_c: &EuclidVector,
    n: u32
) {
    let n = n as f64;
    let polar_vector = euclid_vector_to_polar(vector);
    let r = polar_vector.r;
    let phi = polar_vector.phi;
    let theta = polar_vector.theta;
    let rn = r.powf(n);
    vector.x = rn * (n * theta).sin() * (n * phi).cos() + vector_c.x;
    vector.y = rn * (n * theta).sin() * (n * phi).sin() + vector_c.y;
    vector.z = rn * (n * theta).cos() + vector_c.z;
}

fn sqr_vector(vector: &EuclidVector) -> f64 {
    vector.x * vector.x + vector.y * vector.y + vector.z * vector.z
}

fn render(
    pixels: &mut Vec<u8>,
    side: usize,
    power: u32,
    angle_xy: f64,
    angle_xz: f64,
    angle_yz: f64,
    max_iter: u32,
) {
    assert_eq!(pixels.len(), side * side);
    let bailout = 2.0f64.powf(1.0f64 / (power as f64 - 1.0));
    let bailout2 = bailout.powi(2);
    let half_side = side / 2;
    for y in 0..side {
        for x in 0..side {
            let mut sqr_v: Option<f64> = None;
            for z in 0..half_side {
                let mut vector = &mut coords_to_euclid_vector(
                    (x, y, z),
                    (angle_xy, angle_xz, angle_yz),
                    side,
                    bailout,
                );
                let vector_c = vector.clone();
                for _i in 0..max_iter {
                    iterate_euclid_vector(
                        &mut vector,
                        &vector_c,
                        power);
                }
                let sqr = sqr_vector(&vector);
                if sqr <= bailout2 {
                    sqr_v = Some(sqr);
                    break;
                }
            }
            pixels[y * side + x] = match sqr_v {
                Some(s) => ((s / bailout2) * 255f64) as u8,
                None => 255
            };
        }
    }
}

fn write_image(
    filename: &str,
    pixels: &Vec<u8>,
    side: usize,
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);
    encoder.encode(
        pixels.as_slice(),
        side as u32,
        side as u32,
        ColorType::Gray(8)
    )?;
    Ok(())
}

fn main() {
    let power: u32 = 8;
    let angle_xy = 0.0;
    let angle_xz = 0.0;
    let angle_yz = 0.0;
    let side: usize = 960;
    let mut pixels = vec![127u8; side * side];
    render(&mut pixels, side, power, angle_xy, angle_xz, angle_yz, 7);
    write_image("output.png", &pixels, side)
        .expect("Error on writing PNG-file");
}
