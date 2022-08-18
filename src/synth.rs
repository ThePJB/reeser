use crate::fftviewer::*;
use crate::kinput::*;
use crate::kmath::*;
use crate::sound::*;
use crate::krenderer::*;
use crate::keyboard::*;
use crate::envelope::*;
use crate::filter::*;

use ringbuf::Producer;

// there is big dc i probably need to go negatory as well
pub struct Synth {
    pub sound: Sound,
    pub keyboard: Keyboard,
    pub envelope: Envelope,
    pub filter: FilterPlanner,
    pub fft_viewer: FftViewer,

    pub local_mixer: Mixer,

    pub detune: f32,
    pub voices: f32,

    pub any_change: bool,
}

impl Synth {
    pub fn new() -> Synth {
        Synth {
            sound: Sound::new(),
            any_change: false,
            keyboard: Keyboard::new(),
            envelope: Envelope::new(),
            filter: FilterPlanner::new(),
            fft_viewer: FftViewer::new(512),
            local_mixer: Mixer::new(44100.0),
            voices: 3.0,
            detune: 5.0,
        }
    }

    // ok so we need a local mixer
    // maybe using time to keep up to speed? hoopefully it stays in sync
    // maybe I can downsample before going into fft?

    pub fn frame(&mut self, inputs: &FrameInputState, kc: &mut KRCanvas, sound_channel: &mut Producer<SoundMessage>) {

        // ffwd local mixer
        let ticks = 44100.0f64 * inputs.dt;
        for i in 0..ticks as i32 {
            self.fft_viewer.tick(self.local_mixer.tick());
        }

        kc.set_camera(inputs.screen_rect);
        kc.set_depth(1.0);
        kc.set_colour(Vec4::new(0.8, 0.4, 0.2, 1.0));
        kc.rect(inputs.screen_rect);

        let (top, bottom) = inputs.screen_rect.split_ud(0.65);

        let tops = top.split_lrn(4);

        self.envelope.frame(inputs, kc, tops[0]);
        
        let mids = tops[1].split_lrn(3);

        label_slider("voices", mids[0], 1.0, 9.0, &mut self.voices, false, inputs, kc) |
        label_slider("detune", mids[1], 0.0, 316.0, &mut self.detune, false, inputs, kc) |
        label_slider("volume", mids[2], 0.0, 1.0, &mut self.sound.amplitude, false, inputs, kc);

        if self.filter.frame(inputs, kc, tops[2]) {
            self.sound.filter = self.filter;
        };

        self.sound.voices = self.voices as u32;
        self.sound.detune = self.detune;
        
        self.sound.envelope = self.envelope;

        let keyboard_area = bottom;
        let keyboard_events = self.keyboard.frame(inputs, kc, keyboard_area);


        if keyboard_events.len() > 0 {
            println!("keyboard events: {:?}", keyboard_events);
        }

        for ke in keyboard_events {
            if ke.pressed {
                let mut s = self.sound.clone();
                s.freq = ke.freq;
                sound_channel.push(SoundMessage::PlaySound(s, ke.uid));
                self.local_mixer.add_sound(s, ke.uid);
            } else {
                sound_channel.push(SoundMessage::StopSound(ke.uid));
                self.local_mixer.stop_sound(ke.uid);
            }
        }
        // todo update sounds when sliders adn stuff are adjusted as well

        self.fft_viewer.frame(kc, tops[3])

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

pub fn label_slider(label: &str, r: Rect, min: f32, max: f32, val: &mut f32, log: bool, inputs: &FrameInputState, kc: &mut KRCanvas) -> bool {
    kc.set_depth(1.5);
    let r = r.dilate_pc(-0.02);
    let (text, slider_rect) = r.split_ud(0.05);
    kc.text_center(label.as_bytes(), text);
    slider(slider_rect, min, max, val, log, inputs, kc)
}

pub fn slider(r: Rect, min: f32, max: f32, val: &mut f32, log: bool, inputs: &FrameInputState, kc: &mut KRCanvas) -> bool {
    let r = r.fit_aspect_ratio(0.25);

    kc.set_depth(2.0);
    kc.set_colour(Vec4::new(0.2, 0.2, 0.2, 1.0));
    kc.rect(r);
    kc.set_depth(2.1);
    kc.set_colour(Vec4::new(0.9, 0.9, 0.9, 1.0));
    kc.rect(r.fit_aspect_ratio(0.01));
    kc.set_depth(2.2);
    kc.set_colour(Vec4::new(0.7, 0.7, 0.7, 1.0));

    
    let mut slider_t = 0.0f32;
    let change = r.contains(inputs.mouse_pos) && inputs.lmb == KeyStatus::Pressed;
    if change {
        slider_t = unlerp(inputs.mouse_pos.y, r.bot(), r.top());
        if slider_t < 0.01 {
            slider_t = 0.0;
        }
        if slider_t > 1.0 - 0.01 {
            slider_t = 1.0;
        }
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
            slider_t = unlerp(*val, min, max);
        }
    }


    // linear sliders are wrong when not change, but right when change
    // i mean it looks so simple and correct, remap val, but actually it is wrong anyway / i dont understand why its to low top

    let slider_pos = lerp(r.bot(), r.top(), slider_t);
    let rect_ish = r.dilate_pc(-0.05).fit_aspect_ratio(2.0);
    let slider_rect = Rect::new_centered(r.centroid().x, slider_pos, rect_ish.w, rect_ish.h);
    kc.rect(slider_rect);
    kc.set_depth(2.3);
    kc.text_center(format!("{:.2}", *val).as_bytes(), slider_rect);
    change

    // also render text and name of contained value
}