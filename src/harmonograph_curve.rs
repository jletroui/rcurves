use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use ggez::event::{Axis, Button};
use ggez::GameResult;
use ggez::glam::Vec2;
use ggez::graphics::{Color, DrawParam, MeshBuilder};
use crate::interactive_curve::{DrawableMeshFromBuilder, InteractiveCurve};

const GRAY_LEVEL: f32 = 0.1;
const COLOR: Color = Color::new(GRAY_LEVEL, GRAY_LEVEL, GRAY_LEVEL, 0.7);
const PAPERX: usize = 0;
const PAPERY: usize = 1;
const PENX: usize = 2;
const PENY: usize = 3;
const AMP: usize = 0;
const FREQ: usize = 1;
const PHASE: usize = 2;
const DECAY: usize = 3;
const EPSILON: f32 = 0.01;
const PARAM_NAMES: [&'static str; 4] = [
    "< [amp]  freq   phase   decay  >",
    "<  amp  [freq]  phase   decay  >",
    "<  amp   freq  [phase]  decay  >",
    "<  amp   freq   phase  [decay] >",
];

const NB_ITER: u32 = 30000;
const T_STEP: f32 = 0.015;

struct Pendulum {
    amp: f32, // Note: 2 pendulum in the same axis must have the sum of their amp equal 1.0
    freq: f32,
    phase: f32,
    decay: f32, // Damp factor in exp(-decay*t)
}

impl Pendulum {
    fn new(amp: f32, freq: f32, phase: f32, decay: f32) -> Self {
        Self { amp, freq, phase, decay }
    }

    fn position(&self, t: f32) -> f32 {
        self.amp * f32::sin(self.freq * t + self.phase) * f32::exp(-self.decay * t)
    }

    fn param_value(&self, i: usize) -> f32 {
        match i {
            0 => self.amp,
            1 => self.freq,
            2 => self.phase,
            3 => self.decay,
            _ => panic!("Asked for an unknown Pendulum parameter index: {}", i),
        }
    }
}

pub struct Harmonograph {
    pendulums: [Pendulum; 4],
    displayed_param: usize,
    fixing_values: bool,
    values: HashMap<Axis, f32>,
    axis_to_pendulum: HashMap<Axis, usize>,
}

impl Harmonograph {
    pub fn new() -> Self {
        Self {
            pendulums: [
                Pendulum::new(0.25, 7.5, 0.0, 0.0004),
                Pendulum::new(0.25, 4.0, 0.0, 0.0004),
                Pendulum::new(0.75, 1.0, 0.0, 0.0004),
                Pendulum::new(0.75, 2.0, 0.0, 0.0004),
            ],
            displayed_param: AMP,
            fixing_values: false,
            values: HashMap::new(),
            axis_to_pendulum: [
                (Axis::LeftStickX, PAPERX),
                (Axis::LeftStickY, PAPERY),
                (Axis::RightStickX, PENX),
                (Axis::RightStickY, PENY),
            ].iter().cloned().collect(),
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

    fn normalize(value: f32, upper: f32) -> f32 {
        let norm = (value + 1.0) / 2.0;
        return norm * upper;
    }

    fn adjust_amp_for_axis(&mut self, axis: Axis, value: f32) {
        let new_value = Harmonograph::normalize(value, 1.0);

        match axis {
            Axis::LeftStickY => {
                self.pendulums[PAPERY].amp = new_value;
                self.pendulums[PENY].amp = 1.0 - new_value;
            },
            Axis::RightStickX => {
                self.pendulums[PAPERX].amp = new_value;
                self.pendulums[PENX].amp = 1.0 - new_value;
            },
            _ => ()
        }
    }

    fn adjust_freq_for_axis(&mut self, axis: Axis, value: f32) {
        let new_value = Harmonograph::normalize(value, 20.0).round() / 2.0;
        self.pendulums[*self.axis_to_pendulum.get(&axis).unwrap()].freq = new_value;
    }

    fn adjust_phase_for_axis(&mut self, axis: Axis, value: f32) {
        let new_value = Harmonograph::normalize(value, PI / 2.0);
        self.pendulums[*self.axis_to_pendulum.get(&axis).unwrap()].phase = new_value;
    }

    fn adjust_decay_for_axis(&mut self, axis: Axis, value: f32) {
        let new_value = Harmonograph::normalize(value, 0.002);
        self.pendulums[*self.axis_to_pendulum.get(&axis).unwrap()].decay = new_value;
    }
}

impl Display for Harmonograph {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HARMONOGRAPH {} [Paper] x {:<6.4} y {:<6.4} [Pen] x {:<6.4} y {:<6.4}",
            PARAM_NAMES[self.displayed_param],
            self.pendulums[PAPERX].param_value(self.displayed_param),
            self.pendulums[PAPERY].param_value(self.displayed_param),
            self.pendulums[PENX].param_value(self.displayed_param),
            self.pendulums[PENY].param_value(self.displayed_param),
        )
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
            Button::DPadLeft  => if self.displayed_param > 0 { self.displayed_param = self.displayed_param - 1 },
            Button::DPadRight => if self.displayed_param < 3 { self.displayed_param = self.displayed_param + 1 },
            Button::LeftTrigger | Button::RightTrigger => self.fixing_values = true,
            _ => ()
        }
    }

    fn adjust_for_axis(self: &mut Self, axis: Axis, value: f32) {
        self.values.insert(axis, value);
        let all_zeroes = self.values.values().all(|v| v.abs() < EPSILON);
        if self.fixing_values {
            if all_zeroes {
                self.fixing_values = false;
            }
            else {
                return
            }
        }
        else {
            match self.displayed_param {
                AMP => self.adjust_amp_for_axis(axis, value),
                FREQ => self.adjust_freq_for_axis(axis, value),
                PHASE => self.adjust_phase_for_axis(axis, value),
                DECAY => self.adjust_decay_for_axis(axis, value),
                unknown => panic!("Tried to adjust unknown axis {} in Harmonograph", unknown),
            }
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
