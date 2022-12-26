use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use ggez::event::{Axis, Button};
use ggez::GameResult;
use ggez::glam::Vec2;
use ggez::graphics::{Color, DrawParam, MeshBuilder};
use crate::interactive_curve::{DrawableMeshFromBuilder, InteractiveCurve};
use crate::color_picker::{ColorPicker, HSV};

const PAPERX: usize = 0;
const PAPERY: usize = 1;
const PENX: usize = 2;
const PENY: usize = 3;
const AMP: usize = 0;
const FREQ: usize = 1;
const PHASE: usize = 2;
const DECAY: usize = 3;
const START_COLOR: usize = 4;
const END_COLOR: usize = 5;
const EPSILON: f32 = 0.01;
const PARAM_NAMES: [&'static str; 6] = [
    "< [amp]  freq   phase   decay   startColor   endColor >",
    "<  amp  [freq]  phase   decay   startColor   endColor >",
    "<  amp   freq  [phase]  decay   startColor   endColor >",
    "<  amp   freq   phase  [decay]  startColor   endColor >",
    "<  amp   freq   phase   decay  [startColor]  endColor >",
    "<  amp   freq   phase   decay   startColor  [endColor]>",
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
    pinning_values: bool,
    values: HashMap<Axis, f32>,
    axis_to_pendulum: HashMap<Axis, usize>,
    start_color_picker: ColorPicker,
    end_color_picker: ColorPicker,
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
            pinning_values: false,
            values: HashMap::new(),
            axis_to_pendulum: [
                (Axis::LeftStickX, PAPERX),
                (Axis::LeftStickY, PAPERY),
                (Axis::RightStickX, PENX),
                (Axis::RightStickY, PENY),
            ].iter().cloned().collect(),
            start_color_picker: ColorPicker::new(HSV::new(180.0, 0.75, 0.75)),
            end_color_picker: ColorPicker::new(HSV::new(60.0, 0.75, 0.75)),
        }
    }

    fn point(self: &Self, radius_x: f32, radius_y: f32, t: f32) -> Vec2 {
        return Vec2::new(
            radius_x * (self.pendulums[PAPERX].position(t) + self.pendulums[PENX].position(t)),
            radius_y * (self.pendulums[PAPERY].position(t) + self.pendulums[PENY].position(t)),
        )
    }

    fn color(&self, t: f32) -> Color {
        let t = t % (2.0 * PI);
        let interpolation = if t <= PI {
            t / PI
        } else {
            1.0 - (t - PI) / PI
        };
        let start_color = self.start_color_picker.color();
        let end_color = self.end_color_picker.color();
        let interpolate = |start: f32, end: f32| start + interpolation  * (end - start);
        Color::new(
            interpolate(start_color.r, end_color.r),
            interpolate(start_color.g, end_color.g),
            interpolate(start_color.b, end_color.b),
            1.0
        )
    }

    fn normalize(value: f32, upper: f32) -> f32 {
        let norm = (value + 1.0) / 2.0;
        return norm * upper;
    }

    fn adjust_start_color_for_axis(&mut self, axis: Axis, value: f32) {
        match axis {
            Axis::LeftStickX    => self.start_color_picker.adjust_hue(Harmonograph::normalize(value, 359.9)),
            Axis::LeftStickY    => self.start_color_picker.adjust_saturation(1.0 - Harmonograph::normalize(value, 1.0)),
            _ => ()
        }
    }

    fn adjust_end_color_for_axis(&mut self, axis: Axis, value: f32) {
        match axis {
            Axis::LeftStickX    => self.end_color_picker.adjust_hue(Harmonograph::normalize(value, 359.9)),
            Axis::LeftStickY    => self.end_color_picker.adjust_saturation(1.0 - Harmonograph::normalize(value, 1.0)),
            _ => ()
        }
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
        if self.displayed_param < START_COLOR {
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
        else {
            write!(
                f,
                "HARMONOGRAPH {}",
                PARAM_NAMES[self.displayed_param],
            )

        }
    }
}

impl InteractiveCurve for Harmonograph {
    fn meshes(self: &Self, dest: Vec2, size: Vec2) -> GameResult<Vec<DrawableMeshFromBuilder>> {
        let radius = size / 2.0;
        let mut builder = MeshBuilder::new();
        let mut previous_pt = self.point(radius.x, radius.y, 0.0);
        for i in 0..NB_ITER {
            let t = (i as f32) * T_STEP;
            let pt = self.point(radius.x, radius.y, t);
            builder.line(&[previous_pt, pt], 1.0, self.color(t))?;
            previous_pt = pt;
        }
        let mesh = DrawableMeshFromBuilder::new(builder, DrawParam::new().dest(dest));

        match self.displayed_param {
            START_COLOR => {
                let min_size = size.min_element();
                let picker_size = min_size / 3.0;
                let picker_dest = Vec2::new(dest.x - min_size / 4.0, dest.y);
                let start_meshes = self.start_color_picker.meshes(picker_size, picker_dest)?;
                Ok(vec!(mesh, start_meshes))
            }
            END_COLOR => {
                let min_size = size.min_element();
                let picker_size = min_size / 3.0;
                let picker_dest = Vec2::new(dest.x + min_size / 4.0, dest.y);
                let end_meshes = self.end_color_picker.meshes(picker_size, picker_dest)?;
                Ok(vec!(mesh, end_meshes))
            }
            _ => Ok(vec!(mesh))
        }
    }

    fn adjust_for_button(self: &mut Self, btn: Button) {
        match btn {
            Button::DPadLeft  => if self.displayed_param > 0 { self.displayed_param = self.displayed_param - 1 },
            Button::DPadRight => if self.displayed_param < 5 { self.displayed_param = self.displayed_param + 1 },
            Button::LeftTrigger | Button::RightTrigger => self.pinning_values = true,
            Button::South if self.displayed_param == START_COLOR => self.start_color_picker.incr_value(-0.25),
            Button::East if self.displayed_param == START_COLOR => self.start_color_picker.incr_value(0.25),
            Button::South if self.displayed_param == END_COLOR => self.end_color_picker.incr_value(-0.25),
            Button::East if self.displayed_param == END_COLOR => self.end_color_picker.incr_value(0.25),
            _ => ()
        }
    }

    fn adjust_for_axis(self: &mut Self, axis: Axis, value: f32) {
        self.values.insert(axis, value);

        if self.pinning_values {
            let all_zeroes = self.values.values().all(|v| v.abs() < EPSILON);
            if all_zeroes {
                self.pinning_values = false;
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
                START_COLOR => self.adjust_start_color_for_axis(axis, value),
                END_COLOR => self.adjust_end_color_for_axis(axis, value),
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
