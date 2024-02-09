use ggez::graphics::Color;

pub fn normalize(value: f32, upper: f32) -> f32 {
    let norm = (value + 1.0) / 2.0;
    return norm * upper;
}

pub fn interpolate_color(start_color: &Color, end_color: &Color, interpolation: f32) -> Color {
    let interpolate = |start: f32, end: f32| start + interpolation  * (end - start);
    Color::new(
        interpolate(start_color.r, end_color.r),
        interpolate(start_color.g, end_color.g),
        interpolate(start_color.b, end_color.b),
        1.0
    )
}


