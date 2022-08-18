use crate::kmath::*;

use std::collections::HashSet;
use std::collections::HashMap;
use std::time::{SystemTime, Instant, Duration};

use glutin::event::VirtualKeyCode;

use glutin::event::ElementState;
use glutin::event::MouseButton;
use glutin::event::Event;
use glutin::event::WindowEvent::KeyboardInput;
use glutin::event::WindowEvent::MouseInput;
use glutin::event::WindowEvent::CursorMoved;
use glutin::event::WindowEvent::Resized;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyStatus {
    Pressed,
    JustPressed,
    JustReleased,
    Released,
}

// get rid of repeats did i do this already?

#[derive(Clone)]
pub struct FrameInputState {
    pub screen_rect: Rect,
    pub mouse_pos: Vec2,
    pub mouse_delta: Vec2,
    
    pub prev_keys: HashSet<VirtualKeyCode>,
    pub curr_keys: HashSet<VirtualKeyCode>,

    pub lmb: KeyStatus,
    pub rmb: KeyStatus,
    pub mmb: KeyStatus,
    pub t: f64,
    pub dt: f64,
    pub frame: u32,
    pub seed: u32,
}

impl FrameInputState {
    pub fn key_held(&self, keycode: VirtualKeyCode) -> bool {
        self.curr_keys.contains(&keycode)
    }
    pub fn key_rising(&self, keycode: VirtualKeyCode) -> bool {
        self.curr_keys.contains(&keycode) && !self.prev_keys.contains(&keycode)
    }
    pub fn key_falling(&self, keycode: VirtualKeyCode) -> bool {
        !self.curr_keys.contains(&keycode) && self.prev_keys.contains(&keycode)
    }
}

// Its basically just a state machine to go from events to polling behaviour
pub struct EventAggregator {
    xres: f32,
    yres: f32,
    t_last: Instant,
    instant_mouse_pos: Vec2,
    current: FrameInputState,
}

impl EventAggregator {
    pub fn new(xres: f32, yres: f32) -> EventAggregator {
        EventAggregator { 
            xres, 
            yres, 
            t_last: Instant::now(),
            instant_mouse_pos: Vec2::new(0.0, 0.0),
            current: FrameInputState { 
                screen_rect: Rect::new(0.0, 0.0, xres/yres, 1.0, ), 
                mouse_pos: Vec2::new(0.0, 0.0), 
                mouse_delta: Vec2::new(0.0, 0.0), 
                curr_keys: HashSet::new(),
                prev_keys: HashSet::new(),
                lmb: KeyStatus::Released, 
                rmb: KeyStatus::Released, 
                mmb: KeyStatus::Released, 
                t: 0.0,
                dt: 0.0,
                frame: 0,
                seed: SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or(Duration::from_nanos(34123123)).subsec_nanos(),
            }
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>) -> Option<FrameInputState> {
        match event {
            Event::WindowEvent {event, ..} => match event {
                KeyboardInput { 
                    input: glutin::event::KeyboardInput { 
                        virtual_keycode: Some(virtual_code), 
                        state, 
                    ..},
                ..} => {
                    if *state == ElementState::Pressed {
                        self.current.curr_keys.insert(*virtual_code);
                    } else {
                        self.current.curr_keys.remove(virtual_code);
                    }
                },

                MouseInput { button: glutin::event::MouseButton::Left, state, ..} => {
                    if *state == ElementState::Pressed {
                        self.current.lmb = KeyStatus::JustPressed;
                    } else {
                        self.current.lmb = KeyStatus::JustReleased;
                    }
                },
                MouseInput { button: glutin::event::MouseButton::Middle, state, ..} => {
                    if *state == ElementState::Pressed {
                        self.current.mmb = KeyStatus::JustPressed;
                    } else {
                        self.current.mmb = KeyStatus::JustReleased;
                    }
                },
                MouseInput { button: glutin::event::MouseButton::Right, state, ..} => {
                    if *state == ElementState::Pressed {
                        self.current.rmb = KeyStatus::JustPressed;
                    } else {
                        self.current.rmb = KeyStatus::JustReleased;
                    }
                },


                // Mouse motion
                CursorMoved {
                    position: pos,
                    ..
                } => {
                    self.instant_mouse_pos = Vec2::new(pos.x as f32 / self.yres, pos.y as f32 / self.yres);
                },

                // Resize
                Resized(physical_size) => {
                    self.xres = physical_size.width as f32;
                    self.yres = physical_size.height as f32;
                    self.current.screen_rect = Rect::new(0.0, 0.0, self.xres / self.yres, 1.0);
                },


                // (resize and quit need to be handled by the application)
                _ => {},
                
            },
            Event::MainEventsCleared => {
                let t_now = Instant::now();
                let dt = t_now.duration_since(self.t_last).as_secs_f64();
                self.current.dt = dt;
                self.current.t += dt;
                self.t_last = t_now;
                self.current.frame += 1;
                self.current.mouse_delta = self.instant_mouse_pos - self.current.mouse_pos;
                self.current.mouse_pos = self.instant_mouse_pos;
                let state = self.current.clone();
                self.current.prev_keys = self.current.curr_keys.clone();
                self.current.seed = khash(self.current.seed * 196513497);
                self.current.lmb = match self.current.lmb {KeyStatus::JustPressed | KeyStatus::Pressed => KeyStatus::Pressed, KeyStatus::JustReleased | KeyStatus::Released => KeyStatus::Released};
                self.current.mmb = match self.current.mmb {KeyStatus::JustPressed | KeyStatus::Pressed => KeyStatus::Pressed, KeyStatus::JustReleased | KeyStatus::Released => KeyStatus::Released};
                self.current.rmb = match self.current.rmb {KeyStatus::JustPressed | KeyStatus::Pressed => KeyStatus::Pressed, KeyStatus::JustReleased | KeyStatus::Released => KeyStatus::Released};

                return Some(state);
            },
            _ => {},
        }

        None
    }
}