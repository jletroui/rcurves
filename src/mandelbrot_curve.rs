use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::panic;
use std::time::Instant;
use ggez::event::{Axis, Button, MouseButton};
use ggez::{Context, GameResult};
use ggez::glam::{DVec2, Vec2};
use ggez::glam::i32::IVec2;
use ggez::graphics::{Color, DrawMode, DrawParam, Image as GImage, ImageFormat, MeshBuilder, Rect};
use ggez::input::keyboard::{KeyCode, KeyInput};
use rayon::prelude::*;
use crate::utils;
use crate::color_picker::{ColorPicker, HSV};
use crate::interactive_curve::{DrawData, InteractiveCurve};
use crate::interactive_curve::DrawData::{Image, Meshes};

// Inspiration: http://paulbourke.net/fractals/peterdejong/

const SHOW_KNOWN_CIRCLES: bool = false;
const ZOOM_PAN: usize = 0;
const MAX_ITERATIONS: usize = 1;
const DERIVATIVE_EPSILON: f64 = 0.0001;
const ZERO: f64 = 1e-17;
const OUT_COLOR: usize = 2;
const ALMOST_IN_COLOR: usize = 3;
const PARAM_NAMES: [&'static str; 4] = [
    "< [pan]  iter   outColor   almostInColor >",
    "<  pan  [iter]  outColor   almostInColor >",
    "<  pan   iter  [outColor]  almostInColor >",
    "<  pan   iter   outColor  [almostInColor]>",
];
const PAN_STEP_PCT: f64 = 0.1;
const ZOOM_STEP_PCT: f64 = 0.25;
const VALUE_EPSILON: f32 = 0.01;
const DEFAULT_MAX_ITERATIONS: usize = 100;
const ESCAPE_RADIUS: f64 = 2.;
const INCREASE_MAX_PERIODICITY_AFTER_CYCLES: i32 = 10;
const DEFAULT_BOX_LEFT_X: f64 = -2.;
const DEFAULT_BOX_RIGHT_X: f64 = 0.5;
// https://en.wikibooks.org/wiki/Fractals/Iterations_in_the_complex_plane/Mandelbrot_set/mandelbrot
const KNOW_CIRCLES: [KnownCircle; 3] = [
    // KnownCircle::new(DVec2::new(-0.11, 0.0), 0.63 * 0.63, 0.1), // Main cardioid
    // KnownCircle::new(DVec2::new(0.0, 0.25), 0.35 * 0.35, 2.), // Main cardioid
    // KnownCircle::new(DVec2::new(0.0, -0.25), 0.35 * 0.35, 2.), // Main cardioid
    // KnownCircle::new(DVec2::new(-1.0, 0.0), 0.25 * 0.25, 2.),  // Period 2 bulb
    KnownCircle::new(DVec2::new(-0.125, 0.744), 0.092 * 0.092, 2.),
    KnownCircle::new(DVec2::new(-0.125, -0.744), 0.092 * 0.092, 2.),
    KnownCircle::new(DVec2::new(-1.308, 0.0), 0.058 * 0.058, 2.),
];

struct IterationResult {
    iterations: usize,
    smooth: f64,
    computed: usize,
}

struct KnownCircle {
    center: DVec2,
    radius2: f64,
    x_limit: f64,
}

impl KnownCircle {
    const fn new(center: DVec2, radius2: f64, x_limit: f64) -> Self {
        Self {
            center,
            radius2,
            x_limit,
        }
    }

    fn is_part(self: &Self, c: DVec2) -> bool {
        let dx = c.x - self.center.x;
        let dy = c.y - self.center.y;

        dx * dx + dy * dy < self.radius2 && c.x < self.x_limit
    }

    fn is_known_member(c: DVec2) -> bool {
        // At 100 iterations, pixels from 50% to 38%, iterations from 14% to 3.2%, time from 1200ms to 1300ms
        for circle in KNOW_CIRCLES {
            if circle.is_part(c) {
                return true;
            }
        }

        false
    }
}

struct ViewBox {
    screen_min_x_i: i32,
    // screen_max_x_i: i32,
    screen_min_y_i: i32,
    // screen_max_y_i: i32,
    screen_center: Vec2,
    screen_size_i: IVec2,
    box_center: DVec2,
    box_screen_ratio: f64,
    // x_axis_screen_y_coordinate: i32,
    // has_symmetry: bool,
    // draw_upper: bool,
    pixel_count: usize,
}

impl ViewBox {
    fn zero() -> Self {
        Self::from_center_size(Vec2::new(0., 0.), Vec2::new(0., 0.), DVec2::new(0., 0.), 0.)
    }

    fn from_center_size(screen_center: Vec2, screen_size: Vec2, box_center: DVec2, box_size: f64) -> Self {
        let screen_radius = screen_size.x / 2.;
        let box_screen_ratio = box_size / (screen_size.x as f64);
        // let x_axis_screen_y_coordinate = ((0. - box_center.y) / box_screen_ratio + (screen_center.y as f64)) as f32;
        // let has_symmetry = screen_center.y - screen_radius < x_axis_screen_y_coordinate && x_axis_screen_y_coordinate < screen_center.y + screen_radius;
        // let draw_upper = has_symmetry && x_axis_screen_y_coordinate > screen_center.y;
        let screen_size_i = IVec2::new(screen_size.x.round() as i32, screen_size.y.round() as i32);

        return Self {
            screen_min_x_i: (screen_center.x - screen_radius).round() as i32,
            // screen_max_x_i: (screen_center.x + screen_radius).round() as i32,
            screen_min_y_i: (screen_center.y - screen_radius).round() as i32,
            // screen_max_y_i: (screen_center.y + screen_radius).round() as i32,
            screen_center,
            screen_size_i,
            box_center,
            box_screen_ratio,
            // x_axis_screen_y_coordinate: x_axis_screen_y_coordinate.round() as i32,
            // has_symmetry,
            // draw_upper,
            pixel_count: (screen_size_i.x * screen_size_i.y) as usize,
        }
    }

    fn mandel_point_from_index(self: &Self, pixel_index: usize) -> DVec2 {
        let screen_shift_x = (pixel_index as i32) % self.screen_size_i.y;
        let screen_shift_y = (pixel_index as i32) / self.screen_size_i.y;

        self.mandel_point(
            self.screen_min_x_i + screen_shift_x,
            self.screen_min_y_i + screen_shift_y
        )
    }

    fn mandel_point(self: &Self, screen_pixel_x: i32, screen_pixel_y: i32) -> DVec2 {
        DVec2::new(
            ((screen_pixel_x as f64) - (self.screen_center.x as f64)) * self.box_screen_ratio + self.box_center.x,
            ((screen_pixel_y as f64) - (self.screen_center.y as f64)) * self.box_screen_ratio + self.box_center.y,
        )
    }

    fn screen_pixel(self: &Self, mandel_point: DVec2) -> Vec2 {
        // For displaying circles
        Vec2::new(
            ((mandel_point.x - self.box_center.x) / self.box_screen_ratio) as f32 + self.screen_center.x,
            ((mandel_point.y - self.box_center.y) / self.box_screen_ratio) as f32 + self.screen_center.y,
        )
    }

    // fn screen_x_range(self: &Self) -> Range<i32> {
    //     self.screen_min_x_i..self.screen_max_x_i
    // }
    //
    // fn screen_y_range(self: &Self) -> Range<i32> {
    //     // if self.has_symmetry {
    //     //     if self.draw_upper {
    //     //         self.screen_min_y_i..self.x_axis_screen_y_coordinate
    //     //     } else {
    //     //         self.x_axis_screen_y_coordinate..self.screen_max_y_i
    //     //     }
    //     // } else {
    //         self.screen_min_y_i..self.screen_max_y_i
    //     // }
    // }

    // fn symmetry_point(self: &Self, screen_pixel_y: i32) -> Option<i32> {
    //     if !self.has_symmetry || screen_pixel_y == self.x_axis_screen_y_coordinate {
    //         return None
    //     }
    //
    //     let symmetric_y = 2 * self.x_axis_screen_y_coordinate - screen_pixel_y;
    //     if self.screen_min_y_i < symmetric_y && symmetric_y < self.screen_max_y_i {
    //         Some(symmetric_y)
    //     } else {
    //         None
    //     }
    // }
}

struct ParIterResult {
    histogram: Vec<usize>,
    max_smooth: f64,
    computed_iteration_count: i64,
}

impl ParIterResult {
    fn new(max_iterations: usize) -> Self {
        return ParIterResult {
            histogram: vec![0usize; max_iterations],
            max_smooth: 0.,
            computed_iteration_count: 0,
        }
    }

    fn add(mut self, iter_res: IterationResult) -> Self {
        // If point is not part of the Mandelbrot set, ie iterations == max_iterations
        if iter_res.iterations < self.histogram.len() {
            self.histogram[iter_res.iterations] += 1;
            if iter_res.smooth > self.max_smooth {
                self.max_smooth = iter_res.smooth;
            }
        }
        self.computed_iteration_count += iter_res.computed as i64;
        self
    }

    fn combine(mut self, other: &ParIterResult) -> Self {
        for i in 0..self.histogram.len() {
            self.histogram[i] += other.histogram[i];
        }
        if other.max_smooth > self.max_smooth {
            self.max_smooth = other.max_smooth;
        }
        self.computed_iteration_count += other.computed_iteration_count;
        self
    }
}

struct MandelIterator {
    max_iterations: usize,
    inv_log2: f64,
    escape_radius2: f64,
}

impl MandelIterator {
    fn new(max_iterations: usize, inv_log2: f64, escape_radius2: f64) -> MandelIterator {
        MandelIterator {
            max_iterations,
            inv_log2,
            escape_radius2,
        }
    }

    fn iter_to_divergence(&self, c: DVec2) -> IterationResult {
        if KnownCircle::is_known_member(c) {
            return IterationResult {
                iterations: self.max_iterations,
                smooth: 0.,
                computed: 0
            }
        }

        let mut y = c.y;
        let mut y2 = y * y;

        if self.is_part_of_bulb_or_main_cardioid(c, y2) {
            return IterationResult {
                iterations: self.max_iterations,
                smooth: 0.,
                computed: 0
            };
        }

        let mut x = c.x;
        let mut x2 = x * x;
        let mut xy = x * y;
        let mut xd = 1f64;
        let mut yd = 0f64;
        let mut xd2 = 1f64;
        let mut yd2 = 0f64;
        let mut i = 0usize;
        let mut xh = 0f64;
        let mut yh = 0f64;
        let mut iterations_since_periodicity_refresh = 0;
        let mut max_periodicity = 3;
        let mut refresh_periodicity_cycles = 0;

        while i < self.max_iterations && (x2 + y2) < self.escape_radius2 {
            // Interior detection
            if (xd2 + yd2) < DERIVATIVE_EPSILON {
                return IterationResult {
                    iterations: self.max_iterations,
                    smooth: 0.,
                    computed: i
                };
            }

            xd = 2. * (x * xd - y * yd);
            yd = 2. * (x * yd + y * xd);
            xd2 = xd * xd;
            yd2 = yd * yd;
            x = x2 - y2 + c.x;
            y = xy + xy + c.y;
            x2 = x * x;
            y2 = y * y;
            xy = x * y;

            if f64::abs(x - xh) < ZERO && f64::abs(y - yh) < ZERO {
                return IterationResult {
                    iterations: self.max_iterations,
                    smooth: 0.,
                    computed: 0
                };
            }

            if iterations_since_periodicity_refresh == max_periodicity {
                iterations_since_periodicity_refresh = 0;
                xh = x;
                yh = y;

                if refresh_periodicity_cycles == INCREASE_MAX_PERIODICITY_AFTER_CYCLES {
                    refresh_periodicity_cycles = 0;
                    max_periodicity *= 2;
                }
                refresh_periodicity_cycles += 1;
            }
            iterations_since_periodicity_refresh += 1;

            i += 1;
        }

        if i == self.max_iterations {
            return IterationResult {
                iterations: self.max_iterations,
                smooth: 0.,
                computed: i
            };
        }

        // x = x2 - y2 + c.x;
        // y = xy + xy + c.y;
        // x2 = x * x;
        // y2 = y * y;
        // x = x2 - y2 + c.x;
        // y = xy + xy + c.y;
        // x2 = x * x;
        // y2 = y * y;
        let log_mod_z = f64::log10(x2 + y2) * 0.5;
        let nu = f64::log10(log_mod_z * self.inv_log2) * self.inv_log2;
        return IterationResult {
            iterations: i,
            smooth: 1. - nu,
            computed: i
        };
    }

    fn is_part_of_bulb_or_main_cardioid(&self, c: DVec2, y2: f64) -> bool {
        // https://en.wikibooks.org/wiki/Fractals/Iterations_in_the_complex_plane/Mandelbrot_set/mandelbrot
        let x_plus_1 = c.x + 1.;

        // Bulb
        if x_plus_1 * x_plus_1 + y2 < 0.0625 {
            return true;
        }
        let x_minus_quarter = c.x - 0.25;
        let q = x_minus_quarter * x_minus_quarter + y2;

        // Cardioid
        q * (q + x_minus_quarter) < y2 * 0.25
    }
}

pub struct MandelbrotSet {
    iteration_rate: f32,
    compute_time_ms: [u128; 5],
    colors: Vec<Color>,
    pixels: Vec<u8>,
    img: Option<GImage>,
    iteration_counts: Vec<f32>,
    histogram: Vec<usize>,
    box_center: DVec2,
    box_size: f64,
    max_iterations: usize,
    last_size: Vec2,
    last_colors: [Color; 2],
    last_box_center: DVec2,
    last_box_size: f64,
    last_max_iterations: usize,
    last_view_box: ViewBox,
    out_color_picker: ColorPicker,
    almost_in_color_picker: ColorPicker,
    displayed_param: usize,
    values: HashMap<Axis, f32>,
    pinning_values: bool,
    inv_log2: f64,
    escape_radius2: f64,
}

impl MandelbrotSet {
    pub fn new() -> Self {
        let box_size = DEFAULT_BOX_RIGHT_X - DEFAULT_BOX_LEFT_X;
        let max_iterations = DEFAULT_MAX_ITERATIONS;
        Self {
            iteration_rate: 0.,
            compute_time_ms: [0; 5],
            colors: vec![Color::BLACK; max_iterations + 1],
            pixels: vec![],
            img: None,
            iteration_counts: vec![],
            histogram: vec![0usize; max_iterations],
            box_center: DVec2::new(DEFAULT_BOX_LEFT_X + box_size/2., 0.),
            box_size,
            max_iterations,
            last_size: Vec2::new(0., 0.),
            last_colors: [Color::BLACK; 2],
            last_box_center: DVec2::new(0., 0.),
            last_box_size: 0.,
            last_max_iterations: 0,
            last_view_box: ViewBox::zero(),
            out_color_picker: ColorPicker::new(HSV::new(216.0, 0.85, 0.34), 1./3., Vec2::new(-1./4., 0.)),
            almost_in_color_picker: ColorPicker::new(HSV::new(205.0, 0.87, 0.94),1./3., Vec2::new(1./4., 0.)),
            displayed_param: ZOOM_PAN,
            values: HashMap::new(),
            pinning_values: false,
            inv_log2: 1. / f64::log10(2.),
            escape_radius2: ESCAPE_RADIUS * ESCAPE_RADIUS,
        }
    }

    fn iterator(&self) -> MandelIterator {
        MandelIterator::new(self.max_iterations, self.inv_log2, self.escape_radius2)
    }

    fn fill_colors(&mut self) {
        if self.colors.len() != self.max_iterations + 2 {
            self.colors = vec![Color::BLACK; self.max_iterations + 1];
        }
        let histogram_total = self.histogram.iter().fold(0usize, |a, b| a + b) as f32;
        let mut running_total = 0.;
        let ratio = std::f32::consts::PI * (self.max_iterations as f32) / 100.;
        for i in 0..self.max_iterations {
            self.colors[i] = utils::interpolate_color(
                &self.almost_in_color_picker.color(),
                &self.out_color_picker.color(),
                f32::abs(f32::sin(ratio * running_total / histogram_total)),
//                ratio * (i as f32) / (MAX_ITERATIONS as f32)
            );
            running_total += self.histogram[i] as f32;
        }
    }

    fn adjust_pan(&mut self, x_dir: i8, y_dir: i8) {
        let step = self.box_size * PAN_STEP_PCT;
        let translation = DVec2::new((x_dir as f64)*step, (y_dir as f64)*step);
        self.box_center += translation;
    }

    fn adjust_zoom(&mut self, dir: i8) {
        self.box_size *= 1. + (dir as f64) * ZOOM_STEP_PCT;
    }

    fn color_changed(&self) -> bool {
        self.last_colors[0] != self.out_color_picker.color() || self.last_colors[1] != self.almost_in_color_picker.color()
    }

    fn displayed_color_picker(&self) -> Option<&ColorPicker> {
        match self.displayed_param {
            OUT_COLOR => Some(&self.out_color_picker),
            ALMOST_IN_COLOR => Some(&self.almost_in_color_picker),
            _ => None
        }
    }

    fn mut_displayed_color_picker(&mut self) -> Option<&mut ColorPicker> {
        match self.displayed_param {
            OUT_COLOR => Some(&mut self.out_color_picker),
            ALMOST_IN_COLOR => Some(&mut self.almost_in_color_picker),
            _ => None
        }
    }

    fn record_last_values(&mut self, size: Vec2, view_box: ViewBox) {
        self.last_size = size;
        self.last_colors[0] = self.out_color_picker.color();
        self.last_colors[1] = self.almost_in_color_picker.color();
        self.last_box_center = self.box_center;
        self.last_box_size = self.box_size;
        self.last_max_iterations = self.max_iterations;
        self.last_view_box = view_box;
    }

    fn need_recreate_pixel_cache(&self, size: Vec2) -> bool {
        self.last_size != size
    }

    fn need_recompute_iterations(&self, size: Vec2) -> bool {
        self.need_recreate_pixel_cache(size) ||
            self.last_box_center != self.box_center ||
            self.last_box_size != self.box_size ||
            self.last_max_iterations != self.max_iterations
    }

    fn need_recompute_image(&self, size: Vec2) -> bool {
        self.need_recompute_iterations(size) || self.color_changed()
    }

    fn draw_histogram(&self, dest: Vec2, size: Vec2) -> GameResult<DrawData> {
        let palette_size: f32 = 20.;
        let histogram_size: f32 = 200.;
        let mut builder = MeshBuilder::new();
        let step = size.x / (self.colors.len() as f32) ;

        for i in 0..self.colors.len() {
            let x = (i as f32)*step;
            builder.rectangle(DrawMode::fill(), Rect::new(x, -palette_size, step, palette_size), self.colors[i])?;
        }
        let mut prev_point = Vec2::new(step / 2., 0.);
        let hist_max = self.histogram.iter().fold(0, |a, b| if a > *b { a } else { *b }) as f32;
        for i in 1..self.histogram.len() {
            let new_point = Vec2::new(
                (i as f32)*step + step / 2.,
                (self.histogram[i] as f32) * histogram_size / hist_max
            );
            let panic_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                builder.line(&[prev_point, new_point], 1., Color::RED).unwrap();
            }));
            if let Err(err) = panic_result {
                println!("Panic line: ({} {}), ({} {})", prev_point.x, prev_point.y, new_point.x, new_point.y);
                panic::resume_unwind(err);
            }
            prev_point = new_point;
        }

        Ok(Meshes(builder, DrawParam::new().dest(dest - size / 2.).z(1)))
    }

    fn draw_known_circles(&self, dest: Vec2, size: Vec2) -> GameResult<DrawData> {
        let mut builder = MeshBuilder::new();
        let view_box = ViewBox::from_center_size(dest, size, self.box_center, self.box_size);

        for circle in KNOW_CIRCLES {
            let pixel = view_box.screen_pixel(circle.center);
            let radius = (f64::sqrt(circle.radius2) / view_box.box_screen_ratio) as f32;
            builder.circle(DrawMode::stroke(10.), pixel, radius, 1., Color::RED)?;
        }

        Ok(Meshes(builder, DrawParam::new().z(1)))
    }
}

impl Display for MandelbrotSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MANDEL {}  span: {} dwell: {} time: {}ms iter: {}%",
            PARAM_NAMES[self.displayed_param], self.box_size, self.max_iterations, self.compute_time_ms[0], self.iteration_rate * 100.,
        )
    }
}

impl InteractiveCurve for MandelbrotSet {
    fn compute_drawables(&mut self, ctx: &mut Context, dest: Vec2, size: Vec2) -> GameResult<Vec<DrawData>> {
        let start = Instant::now();
        let view_box = ViewBox::from_center_size(dest, size, self.box_center, self.box_size);

        if self.need_recreate_pixel_cache(size) {
            self.pixels = vec![255u8; 4 * view_box.pixel_count];
            self.iteration_counts = vec![0f32; view_box.pixel_count];
        }

        if self.need_recompute_iterations(size) {
            let iterator = self.iterator();
            let final_acc = self.iteration_counts
                .par_iter_mut()
                .enumerate()
                .map(|(i, iter_count)| {
                    let c = view_box.mandel_point_from_index(i);
                    let iteration_result = iterator.iter_to_divergence(c);
                    *iter_count = iteration_result.iterations as f32 + iteration_result.smooth as f32;
                    iteration_result
                })
                .fold(
                    || ParIterResult::new(iterator.max_iterations),
                    |acc, elt| acc.add(elt)
                )
                .reduce(
                    || ParIterResult::new(iterator.max_iterations),
                    |acc, elt| acc.combine(&elt)
                );

            //println!("Max smooth: {}", final_acc.max_smooth);
            self.iteration_rate = (final_acc.computed_iteration_count as f32) / ((self.max_iterations as f32) * size.x * size.y);
            self.histogram = final_acc.histogram;
            self.compute_time_ms[1] = start.elapsed().as_millis();
        }

        if self.need_recompute_image(size) {
            let fill_start = Instant::now();
            self.fill_colors();
            self.pixels
                .par_iter_mut()
                .chunks(4)
                .enumerate()
                .for_each(|(px_index, mut pixel_slice)| {
                    let iteration_count = self.iteration_counts[px_index];
                    let iteration_count_floor = f32::floor(iteration_count) as usize;

                    let color1 = self.colors[iteration_count_floor];
                    let color2 = self.colors[usize::min(iteration_count_floor + 1, self.max_iterations - 1)];
                    let color = utils::interpolate_color(&color1, &color2, iteration_count % 1.).to_rgb();

                    *pixel_slice[0] = color.2;
                    *pixel_slice[1] = color.1;
                    *pixel_slice[2] = color.0;
                });
            self.compute_time_ms[2] = fill_start.elapsed().as_millis();

            self.img = Some(GImage::from_pixels(
                ctx,
                &self.pixels,
                ImageFormat::Bgra8Unorm,
                view_box.screen_size_i.x as u32,
                view_box.screen_size_i.y as u32
            ));
        }

        if self.need_recompute_iterations(size) {
            self.compute_time_ms[0] = start.elapsed().as_millis();
        }

        self.record_last_values(size, view_box);

        match self.mut_displayed_color_picker() {
            Some(picker) => picker.set_view(size, dest),
            None => ()
        }

        let mut result : Vec<DrawData> = vec!();

        match self.displayed_color_picker() {
            Some(picker) => result.push(picker.meshes()?),
            None => ()
        }

        result.push(self.draw_histogram(dest, size)?);

        if SHOW_KNOWN_CIRCLES {
            result.push(self.draw_known_circles(dest, size)?);
        }

        result.push(Image(self.img.as_ref().unwrap(), DrawParam::new().dest(dest - size / 2.)));

        Ok(result)
    }

    fn adjust_for_button(self: &mut Self, btn: Button) {
        match btn {
            Button::DPadLeft  => if self.displayed_param > 0 { self.displayed_param = self.displayed_param - 1 },
            Button::DPadRight => if self.displayed_param < PARAM_NAMES.len() { self.displayed_param = self.displayed_param + 1 },
            Button::LeftTrigger | Button::RightTrigger => self.pinning_values = true,
            Button::DPadUp if self.displayed_param == ZOOM_PAN  => self.adjust_zoom(-1),
            Button::DPadDown if self.displayed_param == ZOOM_PAN => self.adjust_zoom(1),
            Button::South if self.displayed_param == ZOOM_PAN => self.adjust_pan(0, 1),
            Button::North if self.displayed_param == ZOOM_PAN => self.adjust_pan(0, -1),
            Button::West if self.displayed_param == ZOOM_PAN => self.adjust_pan(-1, 0),
            Button::East if self.displayed_param == ZOOM_PAN => self.adjust_pan(1, 0),
            Button::South if self.displayed_param == MAX_ITERATIONS => self.max_iterations /= 2,
            Button::East if self.displayed_param == MAX_ITERATIONS => self.max_iterations *= 2,
            _ => ()
        }

        match self.mut_displayed_color_picker() {
            Some(picker) => picker.adjust_for_button(btn),
            None => (),
        }
    }

    fn adjust_for_axis(self: &mut Self, axis: Axis, value: f32) {
        self.values.insert(axis, value);

        if self.pinning_values {
            let all_zeroes = self.values.values().all(|v| v.abs() < VALUE_EPSILON);
            if all_zeroes {
                self.pinning_values = false;
            }
            else {
                return
            }
        }
        else {
            match self.mut_displayed_color_picker() {
                Some(picker) => picker.adjust_for_axis(axis, value),
                None => (),
            }
        }
    }

    fn adjust_for_mouse_wheel(&mut self, x: f32, y: f32, wheel_y_dir: f32) {
        self.adjust_zoom(-wheel_y_dir.round() as i8);
        self.box_center = self.last_view_box.mandel_point(x.round() as i32, y.round() as i32);
    }

    fn adjust_for_mouse_button_up(self: &mut Self, button: MouseButton, x: f32, y: f32, drag_start: Vec2) {
        match self.mut_displayed_color_picker() {
            Some(picker) => picker.adjust_for_click(button, x, y),
            None => {
                self.box_center += DVec2::new(
                    ((drag_start.x - x) as f64) * self.last_view_box.box_screen_ratio,
                    ((drag_start.y - y) as f64) * self.last_view_box.box_screen_ratio,
                )
            },
        }
    }

    fn adjust_for_key_up(&mut self, input: KeyInput) {
        match input.keycode {
            Some(KeyCode::Up) => self.max_iterations *= 2,
            Some(KeyCode::Down) => self.max_iterations /= 2,
            _ => ()
        }
    }

    fn screenshot_file_name(&self) -> String {
        format!(
            "mandel",
        )
    }
}
