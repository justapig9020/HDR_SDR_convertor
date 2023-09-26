use std::vec;

use anyhow::Result;
use ndarray::{array, ArrayView1};

const LHDR: f32 = 1_000.0;
const LSDR: f32 = 100.0;
const R: usize = 0;
const _G: usize = 1;
const B: usize = 2;
const PIXEL: usize = 2;

fn gamma_correction(x: f32) -> f32 {
    x.powf(1.0 / 2.4)
}

fn luma(pixel: ArrayView1<f32>) -> f32 {
    let coef = array![0.2627, 0.6780, 0.0593];
    //let coef = array![1.0, 0.0, 0.0];
    pixel.dot(&coef)
}

fn tone_mapping_step1(y: f32) -> f32 {
    let p_hdr = 1.0 + 32.0 * (LHDR / 10_000.0).powf(1.0 / 2.4);
    (1.0 + (p_hdr - 1.0) * y).log(p_hdr)
}

fn tone_mapping_step2(yp: f32) -> f32 {
    if 0.0 <= yp && yp <= 0.7399 {
        1.0770 * yp
    } else if 0.7399 < yp && yp <= 0.9909 {
        -1.1510 * yp * yp + 2.7811 * yp - 0.6302
    } else if 0.9909 < yp && yp <= 1.0 {
        0.5 * yp + 0.5
    } else {
        panic!("Invalid value")
    }
}

fn tone_mapping_step3(yc: f32) -> f32 {
    let p_sdr = 1.0 + 32.0 * (LSDR / 10_000.0).powf(1.0 / 2.4);
    (p_sdr.powf(yc) - 1.0) / (p_sdr - 1.0)
}

fn normalize<T>(x: T) -> f32
where
    T: num::Bounded + Into<f32>,
{
    x.into() as f32 / T::max_value().into() as f32
}

pub fn to_sdr<T>(raw: Vec<T>, hight: u32, width: u32) -> Result<Vec<u8>>
where
    T: std::fmt::Debug + num::Bounded + Clone,
    f32: std::convert::From<T>,
{
    println!("{:?}", &raw[0..6]);
    // shape the raw buffer in to a 3D ndarray
    // The shape of the ndarray is (height, width, 3)
    let rgb = ndarray::Array3::<T>::from_shape_vec((hight as usize, width as usize, 3), raw)?;
    println!("rgb {:?}", rgb.shape());
    let normalized_rgb = rgb.mapv(normalize::<T>);
    // apply the gamma correction
    let linear_rgb = normalized_rgb.mapv(gamma_correction);

    let linear_r = linear_rgb.index_axis(ndarray::Axis(PIXEL), R);
    let linear_b = linear_rgb.index_axis(ndarray::Axis(PIXEL), B);

    let y = linear_rgb.map_axis(ndarray::Axis(PIXEL), luma);

    //return Ok(y.mapv(|x| (x * u8::MAX as f32) as u8).into_raw_vec());

    // apply the tone mapping
    let y_sdr = y
        .mapv(tone_mapping_step1)
        .mapv(tone_mapping_step2)
        .mapv(tone_mapping_step3);

    let colour_scaling = y_sdr / (1.1 * &y);

    //let cb_tmo = colour_scaling * (linear_b - y) / 1.8814;
    //let cr_tmo = colour_scaling * (linear_r - y) / 1.4746;
    let cb_tmo = &colour_scaling * (&linear_b - &y) / 1.8814;

    let cr_tmo = &colour_scaling * (&linear_r - &y) / 1.4746;

    // get the max value from cr_tmo
    let max_cr_tmo = cr_tmo.fold(0.0, |max, &x| if x > max { x } else { max });
    let y_tmo = f32::max(1.0 * max_cr_tmo, 0.0);

    let a = 0.2627;
    let b = 0.6780;
    let c = 0.0593;
    let d = 1.8814;
    let e = 1.4746;

    let r = y_tmo + e * &cr_tmo;
    let g = &y - (a * e / b) * &cr_tmo - (c * d / b) * &cb_tmo;
    let b = &y + d * &cb_tmo;
    println!("r {}", r.get((0, 0)).unwrap());
    println!("g {}", g.get((0, 0)).unwrap());
    println!("b {}", b.get((0, 0)).unwrap());

    // let rgb_tmo = ndarray::stack(ndarray::Axis(2), &[r.view(), g.view(), b.view()])?;
    let rgb_tmo = ndarray::stack(ndarray::Axis(2), &[r.view(), g.view(), b.view()])?;
    println!("{:?}", rgb_tmo.shape());
    println!("{:?}", rgb_tmo.get((0, 0, 0)));
    println!("{:?}", rgb_tmo.get((0, 0, 1)));
    println!("{:?}", rgb_tmo.get((0, 0, 2)));

    // apply the gamma correction
    let rgb_tmo = rgb_tmo.mapv(|x| x.powf(2.4));
    let rgb_tmo = rgb_tmo.mapv(|x| (x * u8::MAX as f32) as u8);
    let mut vec = Vec::with_capacity(rgb_tmo.len());
    for i in rgb_tmo.iter() {
        vec.push(*i);
    }
    // let rgb_tmo = rgb_tmo.into_raw_vec();
    let rgb_tmo = vec;
    println!("{:?}", &rgb_tmo[0..6]);
    Ok(rgb_tmo)
}
