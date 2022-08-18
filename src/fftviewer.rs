use crate::kmath::*;
use crate::krenderer::*;
use rustfft::num_complex::ComplexFloat;
use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;

pub fn blackman(n: usize, N: usize) -> f32 {
    let a0 = 0.42;
    let a1 = 0.5;
    let a2 = 0.08;

    a0 - a1 * (2.0*PI*n as f32/N as f32).cos() + a2 * (4.0*PI*n as f32/N as f32).cos()
}

pub struct FftViewer {
    samples: Vec<f32>,
    head: usize,
    downsample: u32,
    ds_counter: u32,
}

// maybe if I zero pad I can get more continuous looking - zero padding -> interpolation

// need to rescale
// dont care pas like 3khz

// i dont think downsample going in is right
// although it should be? i want fs from 44100hz to 6000hz so the fft is displaying on a range from 0 to 3000hz

// remember to window for spectral leakage as well

// should this be log?

impl FftViewer {
    // pref power of 2 len
    pub fn new(len: usize) -> FftViewer {
        FftViewer {
            samples: vec![0.0; len],
            head: 0,
            downsample: 8,
            ds_counter: 0,
        }
    }

    pub fn tick(&mut self, x: f32) {
        if self.ds_counter == 0 {
            self.samples[self.head] = x;
            self.head = (self.head + 1) % self.samples.len();
        }
        self.ds_counter = (self.ds_counter + 1) % self.downsample;
    }

    pub fn frame(&self, kc: &mut KRCanvas, r: Rect) {
        kc.set_depth(1.5);
        kc.set_colour(Vec4::new(0.4, 0.4, 0.6, 1.0));
        kc.rect(r);
        kc.set_depth(1.6);
        kc.set_colour(Vec4::new(1.0, 1.0, 1.0, 1.0));

        let rects = r.split_lrn(self.samples.len() as i32/2);

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.samples.len());
        // let mut buf: Vec<Complex<f32>> = self.samples.iter().enumerate().map(|(i, x)| Complex{re: *x * blackman(i, self.samples.len()), im: 0.0}).collect();
        let mut buf: Vec<Complex<f32>> = self.samples.iter().map(|x| Complex{re: *x, im: 0.0}).collect();
        // for i in 0..self.samples.len() {
        //     buf.push(Complex{re: 0.0, im: 0.0});
        // }
        fft.process(&mut buf);
        
        // and probably need to fftshift as well

        for i in 0..rects.len() {
            let h = buf[i].abs()/buf.len() as f32;
            // let h = (h + 1.0).ln();
            kc.rect(rects[i].child(0.0, 1.0 - h, 1.0, h));
        }
    }
}