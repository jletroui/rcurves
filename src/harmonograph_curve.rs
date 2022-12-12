use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use ggez::event::Button;
use ggez::GameResult;
use ggez::glam::Vec2;
use ggez::graphics::{Color, DrawParam, MeshBuilder};
use crate::mesh_source::{DrawableMeshFromBuilder, MeshSource};

const AMPLITUDE_INCREMENT: f32 = 0.1;
const FREQ_INCREMENT: f32 = 1.0;
const PHASE_INCREMENT: f32 = PI / 18.0;
const DECAY_INCREMENT: f32 = 0.0002;
const GRAY_LEVEL: f32 = 0.3;
const COLOR: Color = Color::new(GRAY_LEVEL, GRAY_LEVEL, GRAY_LEVEL, 0.7);
const AX: usize = 0;
const AY: usize = 1;
const BX: usize = 2;
const BY: usize = 3;

const NB_ITER: u32 = 20000;
const T_STEP: f32 = 0.02;

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
            "[{}] amp: {} freq (A/B): {} phase (X/Y): {} decay (LT/RT): {}",
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
                Pendulum::new(String::from("AX"), 0.25, 7.5, 0.0, 0.0004),
                Pendulum::new(String::from("AY"), 0.25, 4.0, 0.0, 0.0004),
                Pendulum::new(String::from("BX"), 0.75, 1.001, 0.0, 0.0004),
                Pendulum::new(String::from("BY"), 0.75, 2.0, 0.0, 0.0004),
            ],
            displayed_pendulum: 0,
        }
    }

    fn point(self: &Self, radius_x: f32, radius_y: f32, t: f32) -> Vec2 {
        return Vec2::new(
            radius_x * (self.pendulums[AX].position(t) + self.pendulums[BX].position(t)),
            radius_y * (self.pendulums[AY].position(t) + self.pendulums[BY].position(t)),
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

impl MeshSource for Harmonograph {
    fn meshes(self: &Self, size: Vec2) -> GameResult<Vec<DrawableMeshFromBuilder>> {
        let mut builder = MeshBuilder::new();
        builder.line(&self.points(size.x / 2.0, size.y / 2.0), 1.0, COLOR)?;
        let mesh = DrawableMeshFromBuilder::new(builder, DrawParam::default());
        Ok(vec!(mesh))
    }

    fn adjust_for_button(self: &mut Self, btn: Button) {
        match btn {
            Button::DPadDown => self.displayed_pendulum = AY,
            Button::DPadUp => self.displayed_pendulum = AX,
            Button::DPadLeft => self.displayed_pendulum = BY,
            Button::DPadRight => self.displayed_pendulum = BX,
            Button::LeftTrigger => self.pendulums[self.displayed_pendulum].decay -= DECAY_INCREMENT,
            Button::RightTrigger => self.pendulums[self.displayed_pendulum].decay += DECAY_INCREMENT,
            Button::South => self.pendulums[self.displayed_pendulum].freq -= FREQ_INCREMENT,
            Button::East => self.pendulums[self.displayed_pendulum].freq += FREQ_INCREMENT,
            Button::West => self.pendulums[self.displayed_pendulum].phase -= PHASE_INCREMENT,
            Button::North => self.pendulums[self.displayed_pendulum].phase += PHASE_INCREMENT,
            // Button::LeftTrigger2 => self.pendulums[self.displayed_pendulum].amp -= DECAY_INCREMENT,
            // Button::RightTrigger2 => self.pendulums[self.displayed_pendulum].amp += DECAY_INCREMENT,
            _ => ()
        }
    }

    fn screenshot_file_name(&self) -> String {
        format!(
            "armono_ax_amp{}_freq{}_ph{}_dec{}_ay_amp{}_freq{}_ph{}_dec{}_bx_amp{}_freq{}_ph{}_dec{}_by_amp{}_freq{}_ph{}_dec{}",
            self.pendulums[AX].amp, self.pendulums[AX].freq, self.pendulums[AX].phase, self.pendulums[AX].decay,
            self.pendulums[AY].amp, self.pendulums[AY].freq, self.pendulums[AY].phase, self.pendulums[AY].decay,
            self.pendulums[BX].amp, self.pendulums[BX].freq, self.pendulums[BX].phase, self.pendulums[BX].decay,
            self.pendulums[BY].amp, self.pendulums[BY].freq, self.pendulums[BY].phase, self.pendulums[BY].decay
        )
    }
}
