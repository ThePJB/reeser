use crate::kinput::*;
use crate::kmath::*;
use crate::krenderer::*;

#[derive(Clone, Copy)]
pub struct Sound {
    pub freq: f32,
    pub A: f32,
    pub D: f32,
    pub S: f32,
    pub R: f32,

    pub fmod_freq: f32,
    pub fmod_amt: f32,

    pub amplitude: f32,
    pub amp_lfo_freq: f32,
    pub amp_lfo_amount: f32,

    pub duration: f32,
}

impl Sound {
    pub fn new() -> Sound {
        Sound {
            freq: 440.0,
            A: 1.0,
            D: 1.0,
            S: 1.0,
            R: 1.0,
            fmod_freq: 100.0,
            fmod_amt: 0.0,
            amplitude: 0.1,
            amp_lfo_freq: 20.0,
            amp_lfo_amount: 0.0, 
            duration: 1.0,
        }
    }
}

pub struct Synth {
    pub sound: Sound,
    pub any_change: bool,
}

impl Synth {
    pub fn new() -> Synth {
        Synth {
            sound: Sound::new(),
            any_change: false,
        }
    }

    pub fn frame(&mut self, inputs: &FrameInputState, kc: &mut KRCanvas) {
        kc.set_camera(inputs.screen_rect);
        let w = 11;
        let h = 1;
        self.any_change |= slider(inputs.screen_rect.grid_child(0, 0, w, h), 50.0, 10000.0, &mut self.sound.freq, false, inputs, kc);
        self.any_change |= slider(inputs.screen_rect.grid_child(1, 0, w, h), 0.0, 1.0, &mut self.sound.A, false, inputs, kc);
        self.any_change |= slider(inputs.screen_rect.grid_child(2, 0, w, h), 0.0, 1.0, &mut self.sound.D, false, inputs, kc);
        self.any_change |= slider(inputs.screen_rect.grid_child(3, 0, w, h), 0.0, 1.0, &mut self.sound.S, false, inputs, kc);
        self.any_change |= slider(inputs.screen_rect.grid_child(4, 0, w, h), 0.0, 1.0, &mut self.sound.R, false, inputs, kc);
        self.any_change |= slider(inputs.screen_rect.grid_child(5, 0, w, h), 0.0, 1000.0, &mut self.sound.fmod_freq, true, inputs, kc);
        self.any_change |= slider(inputs.screen_rect.grid_child(6, 0, w, h), 0.0, 1.0, &mut self.sound.fmod_amt, false, inputs, kc);
        self.any_change |= slider(inputs.screen_rect.grid_child(7, 0, w, h), 0.0, 100.0, &mut self.sound.amp_lfo_freq, true, inputs, kc);
        self.any_change |= slider(inputs.screen_rect.grid_child(8, 0, w, h), 0.0, 1.0, &mut self.sound.amp_lfo_amount, false, inputs, kc);
        self.any_change |= slider(inputs.screen_rect.grid_child(9, 0, w, h), 0.0, 10.0, &mut self.sound.duration, false, inputs, kc);
        self.any_change |= slider(inputs.screen_rect.grid_child(10, 0, w, h), 0.0, 1.0, &mut self.sound.amplitude, false, inputs, kc);





    }
}

// so remapping the exponential
// 10, 1000, 0.5 => 100
// 10, 1000, 0.99 => 1000
// 10, 1000, 0.01 => 11

// min + (max - min) * t // linear
// min + 2^t(log2 max)
// min + f((max - min), t) s.t. if t = 0, = 0 t = 1, 1

// f(min, max, t) s.t. t = 0 min, t = 1 max, t 0.5 log_min max 

fn slider(r: Rect, min: f32, max: f32, val: &mut f32, log: bool, inputs: &FrameInputState, kc: &mut KRCanvas) -> bool {
    kc.set_depth(2.0);
    kc.set_colour(Vec4::new(0.2, 0.2, 0.2, 1.0));
    kc.rect(r);
    kc.set_depth(2.1);
    kc.set_colour(Vec4::new(0.9, 0.9, 0.9, 1.0));
    kc.rect(r.fit_aspect_ratio(0.001));
    kc.set_depth(2.2);
    kc.set_colour(Vec4::new(0.7, 0.7, 0.7, 1.0));
    
    let mut slider_t = 0.0f32;
    let change = r.contains(inputs.mouse_pos) && inputs.lmb == KeyStatus::Pressed;
    if change {
        slider_t = unlerp(inputs.mouse_pos.y, r.bot(), r.top());
        if log {
            *val = min + 2.0f32.powf(slider_t * (max - min).log2()) - 1.0;
        } else {
            *val = lerp(min, max, slider_t);
        }
    } else {
        if log {
            // slider t is inverse of that log formula
            slider_t = (*val + 1.0 - min).log2() / (max - min).log2();
        } else {
            // slider t is linear inverse thing
            slider_t = remap(*val, min, max, r.top(), r.bot());
        }
    }

    let slider_pos = lerp(r.bot(), r.top(), slider_t);
    let rect_ish = r.dilate_pc(-0.05).fit_aspect_ratio(2.0);
    let slider_rect = Rect::new_centered(r.centroid().x, slider_pos, rect_ish.w, rect_ish.h);
    kc.rect(slider_rect);
    kc.set_depth(2.3);
    kc.text_center(format!("{:.1}", *val).as_bytes(), slider_rect);
    change

    // also render text and name of contained value
}