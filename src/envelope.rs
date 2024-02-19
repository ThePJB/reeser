use crate::krenderer::*;
use crate::kinput::*;
use crate::kmath::*;
use crate::synth::*;

#[derive(Clone, Copy)]
pub struct Envelope {
    pub a: f32,
    pub d: f32,
    pub s: f32,
    pub r: f32,
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope { a: 0.2, d: 0.2, s: 0.7, r: 0.3 }
    }

    // returns modification
    pub fn frame(&mut self, inputs: &FrameInputState, kc: &mut KRCanvas, rect: Rect) -> bool {
        kc.set_depth(1.1);
        kc.set_colour(Vec4::new(0.4, 0.6, 0.4, 1.0));
        kc.rect(rect);
        kc.set_depth(1.2);
        kc.set_colour(Vec4::new(1.0, 1.0, 1.0, 1.0));
        let (text, sliders) = rect.split_ud(0.15);
        kc.text_center("envelope".as_bytes(), text);

        let sliders = sliders.split_lrn(4);
        label_slider("A", sliders[0].dilate_pc(-0.05), 0.0, 1.0, &mut self.a, false, inputs, kc) |
        label_slider("D", sliders[1].dilate_pc(-0.05), 0.0, 1.0, &mut self.d, false, inputs, kc) |
        label_slider("S", sliders[2].dilate_pc(-0.05), 0.0, 1.0, &mut self.s, false, inputs, kc) |
        label_slider("R", sliders[3].dilate_pc(-0.05), 0.0, 1.0, &mut self.r, false, inputs, kc)
    }

    // maybe needs to smoothstep instead of lerp
    // no help
    // plot values maybe
    // doesnt go negative or anything does it?
    // lol can we do some sheesh filters, nonlinear filters, averager, moog ladder?? might be easy to parameterize

    pub fn amplitude(&self, curr_sample: u32, sample_rate: u32, released_sample: Option<u32>) -> f32 {
        // +1 for useful recursion
        let A = self.a * sample_rate as f32;
        let D = self.d * sample_rate as f32;
        let S = self.s;
        let R = self.r * sample_rate as f32;

        if let Some(released_on) = released_sample {
            let num_released = curr_sample - released_on;
            let release_value = self.amplitude(released_on, sample_rate, None);
            return lerp(release_value, 0.0, num_released as f32 / R);
        }
        if curr_sample as f32 <= A {
            return lerp(0.0, 1.0, curr_sample as f32 / A);
        }
        if curr_sample as f32 <= D + A {
            return lerp(1.0, S, (curr_sample as f32 - A)/D);
        }
        S
    }
}

#[test]
fn test_env() {
    let env = Envelope {a: 1.0, d: 1.0, s: 0.5, r: 1.0};
    assert_eq!(env.amplitude(0, 1000, None), 0.0);
    assert_eq!(env.amplitude(1000, 1000, None), 1.0);
    assert_eq!(env.amplitude(500, 1000, None), 0.5);
    assert_eq!(env.amplitude(1000, 1000, Some(500)), 0.25);
    assert_eq!(env.amplitude(1500, 1000, Some(500)), 0.0);
    
    assert_eq!(env.amplitude(3000, 1000, Some(3000)), 0.5);

    assert_eq!(env.amplitude(2000, 1000, None), 0.5);   
    assert_eq!(env.amplitude(1100, 1000, None), 0.95);  // l 0.5 r 0.75 // str8 to 0.5
    assert_eq!(env.amplitude(1500, 1000, None), 0.75);  // l 0.5 r 0.75
}