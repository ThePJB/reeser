use std::f32::consts::PI;

use crate::krenderer::*;
use crate::kinput::*;
use crate::kmath::*;
use crate::synth::*;
use rustfft::{FftPlanner, num_complex::Complex};

use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::view::ContinuousView;
use plotlib::style::{PointMarker, PointStyle};



pub fn blackmanize(w: &mut Vec<f32>) {
    let N = w.len();
    for n in 0..N {
        let a0 = 0.42;
        let a1 = 0.5;
        let a2 = 0.08;

        let wn = a0 - a1 * (2.0*PI*n as f32/N as f32).cos() + a2 * (4.0*PI*n as f32/N as f32).cos();
        w[n] *= wn;
    }
}

// FIR filter
#[derive(Clone)]
pub struct Filter {
    len: usize,
    head: usize,
    coeffs: Vec<f32>,
    samples: Vec<f32>,
}

impl Filter {
    pub fn new() -> Filter {
        let coeffs = vec![-0.000000, 0.000016, 0.000055, 0.000092, 0.000099, 0.000062, 0.000000, -0.000028, 0.000076, 0.000431, 0.001127, 0.002181, 0.003483, 0.004785, 0.005721, 0.005890, 0.004985, 0.002929, 0.000000, -0.003123, -0.005419, -0.005694, -0.002844, 0.003849, 0.014463, 0.028291, 0.043872, 0.059199, 0.072072, 0.080528, 0.083237, 0.079782, 0.070741, 0.057562, 0.042256, 0.026988, 0.013662, 0.003600, -0.002633, -0.005216, -0.004910, -0.002798, 0.000000, 0.002562, 0.004305, 0.005019, 0.004804, 0.003955, 0.002830, 0.001738, 0.000879, 0.000327, 0.000056, -0.000020, 0.000000, 0.000039, 0.000054, 0.000040, 0.000014, -0.000000,];
        Filter {
            len: 60,
            head: 0,
            coeffs,
            samples: vec![0.0; 60],
        }
    }
    pub fn lowpass(len: usize, fs: f32, fc: f32) -> Filter {
        let mut ideal = vec![Complex{ re: 0.0f32, im: 0.0f32 }; len];
        let n_ones = (len as f32 * (fc / fs)) as usize;
        for i in 0..n_ones {
            ideal[i] = Complex{re: 1.0, im: 0.0f32};
        }
        for i in len - n_ones..len {
            ideal[i] = Complex{re: 1.0, im: 0.0f32};
        }

        // this gain is a bit low
        // probably does need fftshift
        // what baout no blackman

        // ok kinda my bad for not hooking up the inputs. ideal should be complex.

        // oh sheesh need to set for other side as well?
        // then take ifft of ideal
        // then window ideal (pointwise mul)
        // real to real???
        // flip it maybe just do complex to complex and chuck out...

        //lol how to actually validate

        // power of 2 fft is best, could restrict filter length to that..
        // yea dunno what youre playing at, should maek my own.....
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_inverse(len);
        fft.process(&mut ideal);

        // so this sheesh is wasteful of real-complex, its symmetric after all or should be
        // not sure about fftshift or whatever
        // gonna need to plot
        let mut coeffs: Vec<f32> = ideal.iter().map(|x| x.re / len as f32).collect();
        // fftshift. nup wasnt it lmao
        // i think im doing this wrong anyway, meant to swap from ends.? idk
        for i in 0..len/2 {
            coeffs.swap(i, i + len/2);
        }
        blackmanize(&mut coeffs);
        let coeffs_sum: f32 = coeffs.iter().sum();
        for i in 0..len {
            coeffs[i] /= coeffs_sum;
        }
        // hmm suspicious lack of negatives

        // and what do you know compiling is starting to take forever too thanks all these libraries


        //yea dis filter aint work
        // probably plot the impulse response to see. now i need a plotting library lol. is it because i iant fft shift?

        Filter {
            len,
            head: 0,
            coeffs,
            samples: vec![0.0; len],

        }
    }

    pub fn tick(&mut self, sample: f32) -> f32 {
        let mut acc = 0.0;
        self.head = (self.head + 1) % self.len;
        self.samples[self.head] = sample;
        for i in 0..self.len {
            acc += self.coeffs[i] * self.samples[((self.head + self.len - i) % self.len)];
        }
        acc
    }
}

#[derive(Clone, Copy)]
pub struct FilterPlanner {
    pub fs: f32,
    pub fc: f32,
    pub len: f32,
}

impl FilterPlanner {
    pub fn new() -> FilterPlanner {
        FilterPlanner { fs: 44100.0, fc: 800.0, len: 64.0 }
    }

    pub fn frame(&mut self, inputs: &FrameInputState, kc: &mut KRCanvas, rect: Rect) -> bool {
        kc.set_colour(Vec4::new(0.6, 0.4, 0.4, 1.0));
        kc.set_depth(1.1);
        kc.rect(rect);

        let (t, b) = rect.split_ud(0.15);
        kc.set_colour(Vec4::new(1.0, 1.0, 1.0, 1.0));
        kc.set_depth(1.2);
        kc.text_center("filter".as_bytes(), t);

        let (l, r) = b.split_lr(0.5);
        // sliders etc
        label_slider("cutoff", l.dilate_pc(-0.05), 50.0, 3000.0, &mut self.fc, true, inputs, kc) |
        label_slider("len", r.dilate_pc(-0.05), 4.0, 512.0, &mut self.len, false, inputs, kc)

    }

    pub fn lowpass(&self) -> Filter {
        Filter::lowpass(self.len as usize, self.fs, self.fc) 
    }
}