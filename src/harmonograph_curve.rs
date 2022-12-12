use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use ggez::event::Button;
use ggez::GameResult;
use ggez::glam::Vec2;
use ggez::graphics::{Color, DrawParam, MeshBuilder};
use crate::interactive_curve::{DrawableMeshFromBuilder, InteractiveCurve};

const AMPLITUDE_INCREMENT: f32 = 0.1;
const FREQ_INCREMENT: f32 = 1.0;
const PHASE_INCREMENT: f32 = PI / 18.0;
const DECAY_INCREMENT: f32 = 0.0002;
const GRAY_LEVEL: f32 = 0.1;
const COLOR: Color = Color::new(GRAY_LEVEL, GRAY_LEVEL, GRAY_LEVEL, 0.7);
const PAPERX: usize = 0;
const PAPERY: usize = 1;
const PENX: usize = 2;
const PENY: usize = 3;

const NB_ITER: u32 = 30000;
const T_STEP: f32 = 0.015;

struct Pendulum {
    name: String,
    amp: f32, // Note: 2 pendulum in the same axis must have the sum of their amp equal 1.0
    freq: f32,
    phase: f32,
    decay: f32, // Damp factor in exp(-decay*t)
}

impl Pendulum {
    fn new(name: String, amp: f32, freq: f32, phase: f32, decay: f32) -> Self {
        Self { name, amp, freq, phase, decay }
    }

    fn position(&self, t: f32) -> f32 {
        self.amp * f32::sin(self.freq * t + self.phase) * f32::exp(-self.decay * t)
    }
}

impl Display for Pendulum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HARMONOGRAPH {:<10} amp (A / B): {:<6.4} freq (L / R): {:<6} phase (LT / RT): {:<6.4} decay (X / Y): {:<6.4}",
            self.name, self.amp, self.freq, self.phase, self.decay
        )
    }
}

pub struct Harmonograph {
    pendulums: [Pendulum; 4],
    displayed_pendulum: usize,
}

impl Harmonograph {
    pub fn new() -> Self {
        Self {
            pendulums: [
                Pendulum::new(String::from("[Paper-X]"), 0.25, 7.5, 0.0, 0.0004),
                Pendulum::new(String::from("[Paper-Y]"), 0.25, 4.0, 0.0, 0.0004),
                Pendulum::new(String::from("[Pen-X]"), 0.75, 1.001, 0.0, 0.0004),
                Pendulum::new(String::from("[Pen-Y]"), 0.75, 2.0, 0.0, 0.0004),
            ],
            displayed_pendulum: 0,
        }
    }

    fn point(self: &Self, radius_x: f32, radius_y: f32, t: f32) -> Vec2 {
        return Vec2::new(
            radius_x * (self.pendulums[PAPERX].position(t) + self.pendulums[PENX].position(t)),
            radius_y * (self.pendulums[PAPERY].position(t) + self.pendulums[PENY].position(t)),
        )
    }

    fn points(self: &Self, radius_x: f32, radius_y: f32) -> Vec<Vec2> {
        (0..NB_ITER).map(|i| self.point(radius_x, radius_y, (i as f32) * T_STEP)).collect()
    }
}

impl Display for Harmonograph {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.pendulums[self.displayed_pendulum].fmt(f)
    }
}

impl InteractiveCurve for Harmonograph {
    fn meshes(self: &Self, size: Vec2) -> GameResult<Vec<DrawableMeshFromBuilder>> {
        let mut builder = MeshBuilder::new();
        builder.line(&self.points(size.x / 2.0, size.y / 2.0), 1.0, COLOR)?;
        let mesh = DrawableMeshFromBuilder::new(builder, DrawParam::default());
        Ok(vec!(mesh))
    }

    fn adjust_for_button(self: &mut Self, btn: Button) {
        match btn {
            Button::DPadUp => self.displayed_pendulum = if self.displayed_pendulum > 0 { self.displayed_pendulum - 1 } else { 0 },
            Button::DPadDown => self.displayed_pendulum = if self.displayed_pendulum < 3 { self.displayed_pendulum + 1 } else { 3 },
            Button::DPadLeft => self.pendulums[self.displayed_pendulum].freq -= FREQ_INCREMENT,
            Button::DPadRight => self.pendulums[self.displayed_pendulum].freq += FREQ_INCREMENT,
            Button::West => self.pendulums[self.displayed_pendulum].decay -= DECAY_INCREMENT,
            Button::North => self.pendulums[self.displayed_pendulum].decay += DECAY_INCREMENT,
            Button::LeftTrigger => self.pendulums[self.displayed_pendulum].phase -= PHASE_INCREMENT,
            Button::RightTrigger => self.pendulums[self.displayed_pendulum].phase += PHASE_INCREMENT,
            Button::South => {
                self.pendulums[self.displayed_pendulum].amp -= AMPLITUDE_INCREMENT;
                self.pendulums[(self.displayed_pendulum + 2) % 4].amp += AMPLITUDE_INCREMENT;
            },
            Button::East => {
                self.pendulums[self.displayed_pendulum].amp += AMPLITUDE_INCREMENT;
                self.pendulums[(self.displayed_pendulum + 2) % 4].amp -= AMPLITUDE_INCREMENT;
            },
            _ => ()
        }
    }

    fn screenshot_file_name(&self) -> String {
        format!(
            "armono_paperx_amp{}_freq{}_ph{}_dec{}_papery_amp{}_freq{}_ph{}_dec{}_penx_amp{}_freq{}_ph{}_dec{}_peny_amp{}_freq{}_ph{}_dec{}",
            self.pendulums[PAPERX].amp, self.pendulums[PAPERX].freq, self.pendulums[PAPERX].phase, self.pendulums[PAPERX].decay,
            self.pendulums[PAPERY].amp, self.pendulums[PAPERY].freq, self.pendulums[PAPERY].phase, self.pendulums[PAPERY].decay,
            self.pendulums[PENX].amp, self.pendulums[PENX].freq, self.pendulums[PENX].phase, self.pendulums[PENX].decay,
            self.pendulums[PENY].amp, self.pendulums[PENY].freq, self.pendulums[PENY].phase, self.pendulums[PENY].decay
        )
    }
}
