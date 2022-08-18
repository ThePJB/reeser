use std::f32::consts::PI;

use crate::kmath::*;
use crate::filter::*;
use crate::envelope::*;

#[derive(Clone, Copy)]
pub struct Sound {
    // separate counters for the saws rather than modding period would make smooth transitions for the randomwalk of frequency
    // still dunno if it sounds reesey lol.
    // but what next, add detune, period sliders etc
    // also envelope control
    // also filter control

    pub freq: f32,
    pub detune: f32,
    pub voices: u32,

    pub envelope: Envelope,
    
    pub amplitude: f32,

    pub filter: FilterPlanner,
}

impl Sound {
    pub fn new() -> Sound {
        Sound {
            freq: 110.0,
            voices: 2,
            detune: 20.0,
            envelope: Envelope::new(),
            amplitude: 0.2,
            filter: FilterPlanner::new(),
        }
    }

    // i like this, arbitrary mutation to immutable method chain. cya builder pattern
    pub fn but(&self, f: fn(&mut Sound)) -> Sound {
        let mut s = self.clone();
        f(&mut s);
        s
    }

    pub fn play(&self, sample_rate: f32, id: u32) -> PlayingSound {
        PlayingSound {
            sample_rate,
            sample_count: 0,
            sample_released: None,
            sound: self.clone(),
            counters: vec![0; self.voices as usize],
            filter: self.filter.lowpass(),
            id,
        }
    }
}

// specify adsr in terms of samples
// sustain is actually height from 0..1 not samples tho


// seems liek its this but fmsynth works??
// is there still munted hf if i send nothing??

// returns ratio
pub fn detune_interval(cents: f32) -> f32 {
    2.0f32.powf(cents / 1200.0)
}

// we have the ratio of max detune
// we have the individual detune gap
// so its the gap to the power something

// maybe even detune should be as odd but ignore one of the - voices

pub fn detune_voice_n(freq: f32, cents: f32, n: i32, k: i32) -> f32 {
    let detune_interval = detune_interval(cents);
    // let detune_gap = (detune_interval * 2.0) / (k - 1) as f32;
    // freq * detune_gap.powf((n - (k/2)) as f32)
    freq * detune_interval.powf(n as f32)
}

#[test]
fn test_detune() {
    assert_eq!(detune_interval(1200.0), 2.0);
    assert_eq!(detune_interval(2400.0), 4.0);

    assert_eq!(detune_voice_n(1000.0, 1200.0, 0, 1), 1000.0);

    assert_eq!(detune_voice_n(1000.0, 1200.0, 0, 3), 500.0);
    assert_eq!(detune_voice_n(1000.0, 1200.0, 1, 3), 1000.0);
    assert_eq!(detune_voice_n(1000.0, 1200.0, 2, 3), 2000.0);

    // yea even is more of a shitshow
    assert_eq!(detune_voice_n(1000.0, 1200.0, 0, 4), 500.0);
    // assert_eq!(detune_voice_n(1000.0, 1200.0, 1, 4), 1000.0);
    // assert_eq!(detune_voice_n(1000.0, 1200.0, 2, 4), 2000.0);
    assert_eq!(detune_voice_n(1000.0, 1200.0, 3, 4), 2000.0);
}

#[derive(Clone)]
pub struct PlayingSound {
    id: u32,
    sample_rate: f32,
    sample_count: u32,
    sample_released: Option<u32>,
    sound: Sound,
    filter: Filter,
    counters: Vec<u32>,
}

impl PlayingSound {
    pub fn tick(&mut self) -> f32 {
        self.sample_count += 1; // warn overflow

        let env_amp = self.sound.envelope.amplitude(self.sample_count, self.sample_rate as u32, self.sample_released);

        // yea aint sound that good, env the filter maybe
        // let t = (self.sample_count as f32 / (self.sample_rate * 0.5)).min(1.0);
        // let pitch_bend_envelope = lerp(1.5, 1.0, t);
        
        let mut acc = 0.0;
        for i in 0..self.sound.voices {
            let k = self.sound.voices;
            let f = if k == 1 {
                self.sound.freq
            } else {
                detune_voice_n(self.sound.freq, self.sound.detune, i as i32, k as i32)
                // let detune_interval = 2.0f32.powf(self.sound.detune / 1200.0);
                // self.sound.freq * detune_interval.powf((k as f32/2.0 - i as f32)/k as f32)
                // self.sound.freq - detune_freq + 2.0 * i as f32 * detune_freq / (k - 1) as f32
            };
            let this_period = (44100.0 / f) as u32;

            // lol can end up with period of zero, woops
            if this_period != 0 {
                self.counters[i as usize] = (self.counters[i as usize] + 1) % this_period;  // mind this division by 0 hey
            }

            acc += self.counters[i as usize] as f32 / this_period as f32;
        }
        acc /= self.sound.voices as f32;
        acc -= 0.5;
        acc *= 2.0;



        let samp = self.sound.amplitude * env_amp * acc;
        let samp = self.filter.tick(samp);
        // if self.sample_count % 2 == 0 {
        //     return 0.0;
        // } else {
        //     return samp;
        // }
        samp

        // samp
        
    }

    pub fn finished(&self) -> bool {
        if let Some(released) = self.sample_released {
            if (self.sample_count - released) as f32 > self.sound.envelope.r * self.sample_rate {
                return true;
            }
        }
        false
    }
}

pub struct Mixer {
    sample_rate: f32,
    channels: Vec<PlayingSound>,
}

impl Mixer {
    pub fn new(sample_rate: f32) -> Mixer {
        Mixer {
            sample_rate,
            channels: Vec::new(),
        }
    }

    pub fn add_sound(&mut self, sound: Sound, id: u32) {
        // try to put it in one with same id. but this restarts. this fixed weird releasy things
        // do with no restart for when synth params change obviously
        // this is probably why weird popping actually
        // fix make unique id per press
        for i in 0..self.channels.len() {
            if self.channels[i].id == id {
                self.channels[i] = sound.play(self.sample_rate, id);
                return;
            }
        }
        

        for i in 0..self.channels.len() {
            if self.channels[i].finished() {
                self.channels[i] = sound.play(self.sample_rate, id);
                return;
            }
        }
        self.channels.push(sound.play(self.sample_rate, id));   // maybe dont need to replace finished ones
    }

    pub fn stop_sound(&mut self, id: u32) {
        for i in 0..self.channels.len() {
            if self.channels[i].id == id {
                self.channels[i].sample_released = Some(self.channels[i].sample_count);
            }
        }
    }

    pub fn tick(&mut self) -> f32 {
        let mut acc = 0.0;
        for i in 0..self.channels.len() {
                if self.channels[i].finished() {
                    // remove gracefully
                } else {
                    acc += self.channels[i].tick()
                }
            }
        acc
    }
}

pub enum SoundMessage {
    PlaySound(Sound, u32),   // u32 is id. also if its already playing just update the sound
    StopSound(u32),
}