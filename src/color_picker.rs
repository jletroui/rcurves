use ggez::GameResult;
use ggez::glam::Vec2;
use ggez::graphics::{Color, DrawMode, DrawParam, MeshBuilder, Rect};
use crate::interactive_curve::DrawableMeshFromBuilder;

const SPACE_SIZE: f32 = 0.75;
const TARGET_SIZE: f32 = 0.02;
const TARGET_TOLERANCE: f32 = 0.0001;
const TARGET_STROKE_WIDTH: f32 = 0.005;
const TARGET_COLOR: Color = Color::new(0.0, 0.0, 0.0, 1.0);
const MARGIN: f32 = 0.05;
const STEPS_H: usize = 360;
const STEPS_V: usize = 64;
const STEPS_X: f32 = SPACE_SIZE / (STEPS_H as f32);
const STEPS_Y: f32 = SPACE_SIZE / (STEPS_V as f32);

pub struct HSV {
    hue: f32,
    saturation: f32,
    value: f32,
}

impl HSV {
    pub fn new(hue: f32, saturation: f32, value: f32) -> HSV {
        if hue < 0.0 || hue >= 360.0 {
            panic!("h must be between 0 and 360");
        }
        HSV{ hue, saturation, value }
    }
}

pub struct ColorPicker {
    current_pick: HSV,
    last_size: f32,
    last_dest: Vec2,
}

impl ColorPicker {
    pub fn new(current_pick: HSV) -> ColorPicker {
        ColorPicker {
            current_pick,
            last_size: 0.0,
            last_dest: Vec2::new(0.0, 0.0),
        }
    }

    fn from_hsv_components(hue: f32, saturation: f32, value: f32) -> Color {
        // https://www.rapidtables.com/convert/color/hsv-to-rgb.html
        let c = value * saturation;
        let m = value * (1.0 - saturation);
        let x = c * (1.0 - f32::abs((hue / 60.0) % 2.0 - 1.0));
        let (r, g, b) = match hue {
            _ if 0.0   <= hue && hue < 60.0  => (c, x, 0.0),
            _ if 60.0  <= hue && hue < 120.0 => (x, c, 0.0),
            _ if 120.0 <= hue && hue < 180.0 => (0.0, c, x),
            _ if 180.0 <= hue && hue < 240.0 => (0.0, x, c),
            _ if 240.0 <= hue && hue < 300.0 => (x, 0.0, c),
            _ if 300.0 <= hue && hue < 360.0 => (c, 0.0, x),
            _ => panic!("h should be between 0 and 360")
        };

        Color::new(r + m, g + m, b + m, 1.0)
    }

    fn from_hsv(hsv: &HSV) -> Color {
        let HSV { hue, saturation, value } = hsv;
        ColorPicker::from_hsv_components(*hue, *saturation, *value)
    }

    fn params(&self, size: f32, dest: Vec2) -> DrawParam {
        let scaling = Vec2::new(size, size);
        let left_top_dest = dest - size / 2.0;
        DrawParam::new().dest(left_top_dest).scale(scaling)
    }

    pub fn meshes(&mut self, size: f32, dest: Vec2) -> GameResult<DrawableMeshFromBuilder> {
        let mut builder = MeshBuilder::new();
        self.last_size = size;
        self.last_dest = dest;

        // Color space
        for hi in 0..STEPS_H {
            for si in 0..STEPS_V {
                let hue = hi as f32;
                let saturation = (si as f32) / (STEPS_V as f32);
                let x = hue * STEPS_X;
                let y = (si as f32) * STEPS_Y;
                let color = ColorPicker::from_hsv_components(hue, saturation, self.current_pick.value);
                builder.rectangle(DrawMode::fill(), Rect::new(x, y, STEPS_X, STEPS_Y), color)?;
            }
        }

        // Target
        let target_center = Vec2::new(self.current_pick.hue / 360.0 * SPACE_SIZE, self.current_pick.saturation * SPACE_SIZE);
        builder.circle(DrawMode::stroke(TARGET_STROKE_WIDTH), target_center, TARGET_SIZE, TARGET_TOLERANCE, TARGET_COLOR)?;
        builder.rectangle(DrawMode::fill(), Rect::new(SPACE_SIZE, (1.0 - self.current_pick.value) * SPACE_SIZE, MARGIN, TARGET_STROKE_WIDTH), TARGET_COLOR)?;

        // Picked color
        let picked_color = ColorPicker::from_hsv(&self.current_pick);
        builder.rectangle(DrawMode::fill(), Rect::new(0.0, SPACE_SIZE + MARGIN, 1.0, 1.0 - SPACE_SIZE - MARGIN), picked_color)?;
        builder.rectangle(DrawMode::fill(), Rect::new(SPACE_SIZE + MARGIN, 0.0, 1.0 - SPACE_SIZE - MARGIN, SPACE_SIZE + MARGIN), picked_color)?;

        Ok(DrawableMeshFromBuilder::new(builder, self.params(size, dest)))
    }

    pub fn color(&self) -> Color {
        ColorPicker::from_hsv(&self.current_pick)
    }

    pub fn adjust_hue(&mut self, hue: f32) {
        self.current_pick.hue = hue;
    }

    pub fn adjust_saturation(&mut self, saturation: f32) {
        self.current_pick.saturation = saturation;
    }

    pub fn adjust_for_click(&mut self, x: f32, y: f32) {
        let left_top_dest = self.last_dest - self.last_size / 2.0;
        let space_area = Rect::new(left_top_dest.x, left_top_dest.y, self.last_size * SPACE_SIZE, self.last_size * SPACE_SIZE);
        if space_area.contains(Vec2::new(x, y)) {
            let diff_x = x - left_top_dest.x;
            let diff_y = y - left_top_dest.y;
            self.adjust_hue(diff_x / (self.last_size * SPACE_SIZE) * 360.0);
            self.adjust_saturation(diff_y / (self.last_size * SPACE_SIZE))
        }
    }

    pub fn incr_value(&mut self, incr: f32) {
        if (self.current_pick.value > 0.0 && incr < 0.0) || (self.current_pick.value < 1.0 && incr > 0.0) {
            self.current_pick.value += incr;
        }
    }
}
