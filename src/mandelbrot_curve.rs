use std::fmt::{Display, Formatter};
use std::panic;
use std::time::Instant;
use ggez::event::MouseButton;
use ggez::{Context, GameResult};
use ggez::glam::{DVec2, Vec2};
use ggez::glam::i32::IVec2;
use ggez::graphics::{Color, DrawMode, DrawParam, Image as GImage, ImageFormat, MeshBuilder};
use ggez::input::keyboard::{KeyCode, KeyInput};
use rayon::prelude::*;
use ggegui::egui;
use ggegui::egui::{RichText, Ui};
use crate::utils;
use crate::color_picker::{ColorPicker, HSV};
use crate::interactive_curve::{DrawData, InteractiveCurve};
use crate::interactive_curve::DrawData::{Image, Meshes};

// Draw constants
const TARGET_SIZE: f32 = 15.;
const DARK_GREY: Color = Color {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.0,
};

// Params constants
const MAX_ITERATIONS_PARAM: usize = 0;
const OUT_COLOR_PARAM: usize = 1;
const ALMOST_IN_COLOR_PARAM: usize = 2;
const ZOOM_STEP_PCT: f64 = 0.25;
const DEFAULT_BOX_LEFT_X: f64 = -2.;
const DEFAULT_BOX_RIGHT_X: f64 = 0.5;
const DEFAULT_SPAN: f64 = DEFAULT_BOX_RIGHT_X - DEFAULT_BOX_LEFT_X;

// Algorithm constants
const EPSILON: f64 = 1e-17;
const DEFAULT_MAX_ITERATIONS: usize = 100;
const ESCAPE_RADIUS: f64 = 2.;
const INCREASE_MAX_PERIODICITY_AFTER_CYCLES: i32 = 10;
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
const REMARKABLE_POINTS: [(DVec2, &str); 7] = [
    (DVec2::new(DEFAULT_BOX_LEFT_X + DEFAULT_SPAN / 2., 0.), "Défaut"),
    (DVec2::new(-1.401155, 0.), "Feigenbaum"),
    (DVec2::new(-0.743643887037151, 0.13182590420533), "Vallée hippocampes"),
    (DVec2::new(-1.749214022, -0.000289489), "Mini mandelbrot à gauche"),
    (DVec2::new(-0.1649200283, -1.0369146835), "En haut, mini Julia"),
    (DVec2::new(-1.4838688322327218, 0.0000000000000003), "Ligne à gauche"),
    (DVec2::new(0.3621185521154, -0.4261009708377), "Cheveux frisés en haut à droite"),
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

#[derive(Clone)]
struct ViewBox {
    screen_min_i: IVec2,
    screen_center: Vec2,
    screen_size_i: IVec2,
    box_center: DVec2,
    box_size: DVec2,
    box_screen_ratio: f64,
    pixel_count: usize,
}

impl ViewBox {
    fn zero() -> Self {
        Self::from_center_size(Vec2::ZERO, Vec2::ZERO, DVec2::ZERO, DVec2::ZERO)
    }

    fn from_center_size(screen_center: Vec2, screen_size: Vec2, box_center: DVec2, box_size: DVec2) -> Self {
        let screen_size_i = screen_size.round().as_ivec2();

        return Self {
            screen_min_i: (screen_center - screen_size / 2.).round().as_ivec2(),
            screen_center,
            screen_size_i,
            box_center,
            box_size,
            box_screen_ratio: box_size.x / (screen_size.x as f64),
            pixel_count: (screen_size_i.x * screen_size_i.y) as usize,
        }
    }

    fn mandel_point_from_index(self: &Self, pixel_index: usize) -> DVec2 {
        let screen_shift_x = (pixel_index as i32) % self.screen_size_i.x;
        let screen_shift_y = (pixel_index as i32) / self.screen_size_i.x;

        self.mandel_point(
            self.screen_min_i.x + screen_shift_x,
            self.screen_min_i.y + screen_shift_y
        )
    }

    fn mandel_point(self: &Self, screen_pixel_x: i32, screen_pixel_y: i32) -> DVec2 {
        DVec2::new(
            ((screen_pixel_x as f64) - (self.screen_center.x as f64)) * self.box_screen_ratio + self.box_center.x,
            ((screen_pixel_y as f64) - (self.screen_center.y as f64)) * self.box_screen_ratio + self.box_center.y,
        )
    }

    fn screen_pixel(self: &Self, mandel_point: &DVec2) -> Vec2 {
        // For displaying circles
        Vec2::new(
            ((mandel_point.x - self.box_center.x) / self.box_screen_ratio) as f32 + self.screen_center.x,
            ((mandel_point.y - self.box_center.y) / self.box_screen_ratio) as f32 + self.screen_center.y,
        )
    }

    fn screen_pixel_index(self: &Self, screen_x: f32, screen_y: f32) -> usize {
        // For displaying selected point info
        ((screen_x - self.screen_min_i.x as f32) + (screen_y - self.screen_min_i.x as f32) * (self.screen_size_i.x as f32)).round() as usize
    }

    fn size_changed(self: &Self, other: &ViewBox) -> bool {
        self.screen_size_i != other.screen_size_i
    }
}

struct ParIterResult {
    histogram: Vec<usize>,
    min_smooth: f64,
    max_smooth: f64,
    computed_iteration_count: i64,
}

impl ParIterResult {
    fn new(max_iterations: usize) -> Self {
        return ParIterResult {
            histogram: vec![0usize; max_iterations],
            min_smooth: 100.,
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
            if iter_res.smooth < self.min_smooth {
                self.min_smooth = iter_res.smooth;
            }
        }
        self.computed_iteration_count += iter_res.computed as i64;
        self
    }

    fn combine(mut self, other: &ParIterResult) -> Self {
        for i in 0..self.histogram.len() {
            self.histogram[i] += other.histogram[i];
        }
        if other.min_smooth < self.min_smooth {
            self.min_smooth = other.min_smooth;
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
    escape_radius2: f64,
}

impl MandelIterator {
    fn new(max_iterations: usize, escape_radius2: f64) -> MandelIterator {
        MandelIterator {
            max_iterations,
            escape_radius2,
        }
    }

    fn iter_to_divergence(&self, c: DVec2) -> IterationResult {
        // https://en.wikibooks.org/wiki/Fractals/Iterations_in_the_complex_plane/Mandelbrot_set/mandelbrot

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
        let mut i = 0usize;
        let mut xh = 0f64;
        let mut yh = 0f64;
        let mut iterations_since_periodicity_refresh = 0;
        let mut max_periodicity = 3;
        let mut refresh_periodicity_cycles = 0;

        while i < self.max_iterations && (x2 + y2) < self.escape_radius2 {
            x = x2 - y2 + c.x;
            y = xy + xy + c.y;
            x2 = x * x;
            y2 = y * y;
            xy = x * y;

            // Cycle detection
            if f64::abs(x - xh) < EPSILON && f64::abs(y - yh) < EPSILON {
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

        if i >= self.max_iterations {
            return IterationResult {
                iterations: self.max_iterations,
                smooth: 0.,
                computed: i
            };
        }

        let nu = f64::log2(f64::log2(x2 + y2) / 2.);
        IterationResult {
            iterations: i,
            smooth: f64::max(0., 1. - nu),
            computed: i
        }
    }

    fn iteration_points(&self, c: DVec2) -> Vec<DVec2> {
        let mut res = vec![c];
        let mut y = c.y;
        let mut y2 = y * y;
        let mut x = c.x;
        let mut x2 = x * x;
        let mut xy = x * y;
        let mut i = 0usize;

        while i < self.max_iterations && (x2 + y2) < self.escape_radius2 {
            x = x2 - y2 + c.x;
            y = xy + xy + c.y;
            x2 = x * x;
            y2 = y * y;
            xy = x * y;

            res.push(DVec2::new(x, y));
            i += 1;
        }

        res
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

#[derive(Clone, Copy)]
struct PointDetail(DVec2, usize);

pub struct MandelbrotSet {
    iteration_rate: f32,
    compute_time_ms: [u128; 5],
    pixels: Vec<u8>,
    img: Option<GImage>,
    iteration_counts: Vec<f32>,
    histogram: Vec<usize>,
    box_center: DVec2,
    box_size: DVec2,
    max_iterations: usize,
    last_colors: [Color; 2],
    last_max_iterations: usize,
    last_view_box: ViewBox,
    out_color_picker: ColorPicker,
    almost_in_color_picker: ColorPicker,
    displayed_param: usize,
    escape_radius2: f64,
    show_histogram: bool,
    drag_translation: Vec2,
    show_point_details: Option<PointDetail>,
    selected_remarkable_point: usize,
}

impl MandelbrotSet {
    pub fn new() -> Self {
        let max_iterations = DEFAULT_MAX_ITERATIONS;
        let default_location = REMARKABLE_POINTS[0].0;
        Self {
            iteration_rate: 0.,
            compute_time_ms: [0; 5],
            pixels: vec![],
            img: None,
            iteration_counts: vec![],
            histogram: vec![0usize; max_iterations],
            box_center: default_location,
            box_size: DVec2::new(DEFAULT_SPAN, DEFAULT_SPAN),
            max_iterations,
            last_colors: [Color::BLACK; 2],
            last_max_iterations: 0,
            last_view_box: ViewBox::zero(),
            out_color_picker: ColorPicker::new("Out", HSV::new(236., 0.96, 0.94), 1./2., Vec2::new(-1./5., 0.)),
            almost_in_color_picker: ColorPicker::new("Almost in", HSV::new(30.0, 0.91, 1.09),1./2., Vec2::new(1./5., 0.)),
            displayed_param: MAX_ITERATIONS_PARAM,
            escape_radius2: ESCAPE_RADIUS * ESCAPE_RADIUS,
            show_histogram: false,
            drag_translation: Vec2::ZERO,
            show_point_details: None,
            selected_remarkable_point: 0,
        }
    }

    fn reset_to_remarkable_point(&mut self) {
        self.box_center = REMARKABLE_POINTS[self.selected_remarkable_point].0;
        self.box_size = DVec2::new(DEFAULT_SPAN, DEFAULT_SPAN);
        self.max_iterations = 100;
    }

    fn iterator(&self) -> MandelIterator {
        MandelIterator::new(self.max_iterations, self.escape_radius2)
    }

    fn adjust_zoom(&mut self, dir: i8) {
        self.box_size *= 1. + (dir as f64) * ZOOM_STEP_PCT;
        self.show_point_details = None;
    }

    fn color_changed(&self) -> bool {
        self.last_colors[0] != self.out_color_picker.color() || self.last_colors[1] != self.almost_in_color_picker.color()
    }

    fn displayed_color_picker(&self) -> Option<&ColorPicker> {
        match self.displayed_param {
            OUT_COLOR_PARAM => Some(&self.out_color_picker),
            ALMOST_IN_COLOR_PARAM => Some(&self.almost_in_color_picker),
            _ => None
        }
    }

    fn displayed_color_picker_mut(&mut self) -> Option<&mut ColorPicker> {
        match self.displayed_param {
            OUT_COLOR_PARAM => Some(&mut self.out_color_picker),
            ALMOST_IN_COLOR_PARAM => Some(&mut self.almost_in_color_picker),
            _ => None
        }
    }

    fn record_last_values(&mut self, view_box: ViewBox) {
        self.last_colors[0] = self.out_color_picker.color();
        self.last_colors[1] = self.almost_in_color_picker.color();
        self.last_max_iterations = self.max_iterations;
        self.last_view_box = view_box;
    }

    fn need_recreate_pixel_cache(&self, view_box: &ViewBox) -> bool {
        self.last_view_box.size_changed(view_box)
    }

    fn need_recompute_iterations(&self, view_box: &ViewBox) -> bool {
        self.need_recreate_pixel_cache(view_box) ||
        self.last_view_box.box_center != self.box_center ||
        self.last_view_box.box_size != self.box_size ||
        self.last_max_iterations != self.max_iterations
    }

    fn need_recompute_image(&self, view_box: &ViewBox) -> bool {
        self.need_recompute_iterations(view_box) ||
        self.color_changed()
    }

    fn draw_histogram(&self, dest: Vec2, size: Vec2) -> GameResult<DrawData> {
        let palette_size: f32 = 20.;
        let histogram_size: f32 = 200.;
        let mut builder = MeshBuilder::new();
        let step = size.x / (self.max_iterations as f32) ;
        let mut prev_point = Vec2::new(step / 2., 0.);
        let hist_max = self.histogram.iter().fold(0, |a, b| if a > *b { a } else { *b }) as f32;
        for i in 1..self.histogram.len() {
            let new_point = Vec2::new(
                (i as f32)*step + step / 2.,
                palette_size + (self.histogram[i] as f32) * histogram_size / hist_max
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

        Ok(Meshes(builder, DrawParam::new().dest(dest - size / 2.)))
    }

    fn draw_center_target(&self, dest: Vec2) -> GameResult<DrawData> {
        let mut builder = MeshBuilder::new();
        builder.line(&[Vec2::new(-TARGET_SIZE, 0.), Vec2::new(TARGET_SIZE, 0.)], 1., Color::RED)?;
        builder.line(&[Vec2::new(0., -TARGET_SIZE), Vec2::new(0., TARGET_SIZE)], 1., Color::RED)?;
        builder.circle(DrawMode::stroke(1.), Vec2::ZERO, TARGET_SIZE, 1., Color::WHITE)?;

        Ok(Meshes(builder, DrawParam::new().dest(dest)))
    }

    fn draw_point_details(&self, view_box: &ViewBox) -> GameResult<DrawData> {
        let mut builder = MeshBuilder::new();
        let iterator = MandelIterator::new(100, self.escape_radius2);
        let c = match self.show_point_details { Some(PointDetail(p, _)) => p, _ => DVec2::ZERO };
        let points: Vec<Vec2> = iterator
            .iteration_points(c)
            .iter()
            .map(|p| view_box.screen_pixel(p))
            .collect();
        if points.len() > 1 {
            builder.line(&points, 2., Color::RED)?;
        }
        builder.circle(DrawMode::fill(), view_box.screen_pixel(&c), 5., 1., Color::RED)?;

        Ok(Meshes(builder, DrawParam::new()))
    }
}

impl Display for MandelbrotSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ensemble de Mandelbrot")
    }
}

impl InteractiveCurve for MandelbrotSet {
    fn update_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Dwell:");
            if ui.button("-").clicked() {
                self.max_iterations /= 2;
            }
            ui.label(format!("{}", self.max_iterations));
            if ui.button("+").clicked() {
                self.max_iterations *= 2;
            }
        });
        ui.checkbox(&mut self.show_histogram, "Histogramme");
        if ui.button(RichText::new("Couleur 1").background_color(self.out_color_picker.color32())).clicked() {
            self.displayed_param = match self.displayed_param {
                OUT_COLOR_PARAM => MAX_ITERATIONS_PARAM,
                _ => OUT_COLOR_PARAM
            };
        }
        if ui.button(RichText::new("Couleur 2").background_color(self.almost_in_color_picker.color32())).clicked() {
            self.displayed_param = match self.displayed_param {
                ALMOST_IN_COLOR_PARAM => MAX_ITERATIONS_PARAM,
                _ => ALMOST_IN_COLOR_PARAM
            };
        }
        ui.horizontal(|ui| {
            ui.label("Départ:");
            egui::ComboBox::from_id_source("remarkable_points")
                .selected_text(format!("{}", REMARKABLE_POINTS[self.selected_remarkable_point].1))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(60.0);
                    REMARKABLE_POINTS.iter().enumerate().for_each(|(i, (_, pt_name))| {
                        ui.selectable_value(&mut self.selected_remarkable_point, i, *pt_name);
                    });
                });
            if ui.button("Aller").clicked() {
                self.reset_to_remarkable_point();
            }
        });
        ui.separator();
        egui::Grid::new("mandel_info")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Position:");
                ui.label(format!("{:.5} , {:.5}", self.box_center.x, self.box_center.y));
                ui.end_row();
                ui.label("Zoom:");
                ui.label(format!("{:.2e}", DEFAULT_SPAN / self.box_size.x));
                ui.end_row();
                ui.label("Temps calcul:");
                ui.label(format!("{} ms", self.compute_time_ms[0]));
                ui.end_row();
                ui.label("Points calculés:");
                ui.label(format!("{:.1} %", self.iteration_rate * 100.));
                ui.end_row();

                match self.show_point_details {
                    Some(pt) => {
                        ui.label("Itérations au point:");
                        ui.label(format!("{}",self.iteration_counts[pt.1]));
                        ui.end_row();
                    },
                    _ => ()
                }
            });
        ui.separator();
        ui.label("[clic droit]: calcul au point");
        ui.label("[Z]: zoomer");
        ui.label("[X]: dézoomer");
        ui.label("[R]: zoom à 0");
        ui.label("[H]: histogramme");
    }

    fn compute_drawables(&mut self, ctx: &mut Context, dest: Vec2, size: Vec2) -> GameResult<Vec<DrawData>> {
        let start = Instant::now();
        let view_box = ViewBox::from_center_size(dest, size, self.box_center, self.box_size);

        if self.need_recreate_pixel_cache(&view_box) {
            self.pixels = vec![255u8; 4 * view_box.pixel_count];
            self.iteration_counts = vec![0f32; view_box.pixel_count];
        }

        if self.need_recompute_iterations(&view_box) {
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

            //println!("Smooth: [{}, {}]", final_acc.min_smooth, final_acc.max_smooth);
            self.iteration_rate = (final_acc.computed_iteration_count as f32) / ((self.max_iterations as f32) * size.x * size.y);
            self.histogram = final_acc.histogram;
            self.compute_time_ms[1] = start.elapsed().as_millis();
        }

        if self.need_recompute_image(&view_box) {
            let fill_start = Instant::now();
            let colors = [self.out_color_picker.color(), Color::WHITE, self.almost_in_color_picker.color(), DARK_GREY, self.out_color_picker.color()];

            self.pixels
                .par_iter_mut()
                .chunks(4)
                .enumerate()
                .for_each(|(px_index, mut pixel_slice)| {
                    let iteration_count = self.iteration_counts[px_index];

                    let color: (u8,u8,u8) =
                        if iteration_count >= (self.max_iterations as f32) {
                            Color::BLACK.to_rgb()
                        } else {
                            let n_colors = colors.len() as f32 - 1.;
                            let interpolation = (iteration_count / 25.) % n_colors;
                            let color1_index = interpolation.floor() as usize;
                            let color1 = &colors[color1_index];
                            let color2 = &colors[color1_index + 1];
                            //let sub_interpolation = interpolation % 1.;
                            // Adjust the subinterpolation so as to decrease emphasis on black and white
                            let sub_interpolation = f32::sin(interpolation * 2. * std::f32::consts::PI / n_colors - std::f32::consts::PI / 2.);
                            let sign: f32 = if color1_index < 2 { 1. } else { -1. };
                            let adder: f32 = if color1_index % 2 == 0 { 1. } else { 0. };
                            utils::interpolate_color(
                                color1,
                                color2,
                                adder + sign * sub_interpolation,
                            ).to_rgb()
                        };

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

        if self.need_recompute_iterations(&view_box) {
            self.compute_time_ms[0] = start.elapsed().as_millis();
        }

        match self.displayed_color_picker_mut() {
            Some(picker) => picker.set_view(size, dest),
            None => ()
        }

        self.record_last_values(view_box.clone());

        let mut result : Vec<DrawData> = vec!();

        match self.show_point_details {
            Some(_) => result.push(self.draw_point_details(&view_box)?),
            _ => ()
        }

        match self.displayed_color_picker() {
            Some(picker) => result.push(picker.meshes()?),
            None => ()
        }

        if self.show_histogram {
            result.push(self.draw_histogram(dest, size)?);
        }

        if self.drag_translation != Vec2::ZERO {
            result.push(self.draw_center_target(dest)?);
        }

        result.push(Image(self.img.as_ref().unwrap(), DrawParam::new().z(-1).dest((dest - size / 2.) + self.drag_translation)));

        Ok(result)
    }

    fn adjust_for_mouse_button_up(self: &mut Self, button: MouseButton, x: f32, y: f32, drag_start: Vec2) {
        self.drag_translation = Vec2::ZERO;
        match self.displayed_color_picker_mut() {
            Some(picker) => picker.adjust_for_click(button, x, y),
            None if button == MouseButton::Left => {
                self.box_center += DVec2::new(
                    ((drag_start.x - x) as f64) * self.last_view_box.box_screen_ratio,
                    ((drag_start.y - y) as f64) * self.last_view_box.box_screen_ratio,
                );
                self.show_point_details = None;
            },
            None if button == MouseButton::Right => {
                self.show_point_details = Some(
                    PointDetail(
                        self.last_view_box.mandel_point(x as i32, y as i32),
                        self.last_view_box.screen_pixel_index(x, y),
                    )
                )
            },
            _ => ()
        }
    }

    fn adjust_for_mouse_drag(&mut self, x: f32, y: f32, drag_start: Vec2) {
        self.drag_translation = Vec2::new(x, y) - drag_start;
        self.show_point_details = None;
    }

    fn adjust_for_mouse_wheel(&mut self, _x: f32, _y: f32, wheel_y_dir: f32) {
        if f32::abs(wheel_y_dir) >= 0.5 {
            self.adjust_zoom((-wheel_y_dir / wheel_y_dir) as i8);
        }
    }

    fn adjust_for_key_up(&mut self, input: KeyInput) {
        match input.keycode {
            Some(KeyCode::Escape) => self.displayed_param = MAX_ITERATIONS_PARAM,
            Some(KeyCode::R) => self.reset_to_remarkable_point(),
            Some(KeyCode::H) => self.show_histogram = !self.show_histogram,
            Some(KeyCode::Z) => self.adjust_zoom(-1),
            Some(KeyCode::X) => self.adjust_zoom(1),
            _ => ()
        }

        match self.displayed_color_picker_mut() {
            Some(picker) => picker.adjust_for_key(input),
            None => (),
        }
    }

    fn screenshot_file_name(&self) -> String {
        format!(
            "mandel_{}", self.max_iterations
        )
    }

    fn name(&self) -> &str {
        "Ensemble Mandelbrot"
    }

    fn inspiration_url(&self) -> &str {
        "https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set"
    }
}
