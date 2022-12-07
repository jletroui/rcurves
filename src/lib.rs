mod point;
mod lissajou_curve;
mod rendering;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, Button, HatState, ButtonEvent};
use piston::input::controller::ControllerHat;
use piston::window::WindowSettings;
use piston::{AdvancedWindow, ButtonArgs, ButtonState, PressEvent};
use rendering::Renderer;

pub fn run() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("Lissajous", [1080, 1080])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut gl = GlGraphics::new(opengl);
    let mut renderer = Renderer::new(); // TODO: comprendre pourquoi mut nécessaire
    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, gl| {
                renderer.render(&args, c, gl)
            });
        }
        if let Some(button) = e.press_args() {
            match button {
                Button::Hat(ControllerHat { id: _, state: HatState::Up, which: _ })=> {
                    window.set_title(String::from("Up"))
                }
                Button::Hat(ControllerHat { id: _, state: HatState::Down, which: _ })=> {
                    window.set_title(String::from("Down"))
                }
                Button::Hat(_) => {
                    window.set_title(String::from("Hat"))
                }
                Button::Controller(_) => {
                    window.set_title(String::from("Controller"))
                }
                Button::Keyboard(_) => {
                    window.set_title(String::from("Keyboard"))
                }
                Button::Mouse(_) => {
                    window.set_title(String::from("Mouse"))
                }
            }
        }
    }
}
