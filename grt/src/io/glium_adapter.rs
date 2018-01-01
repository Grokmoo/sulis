use std::cell::{Ref, RefCell};
use std::rc::Rc;

use config::CONFIG;
use io::{InputAction, KeyboardEvent, IO};
use io::keyboard_event::Key;
use resource::ResourceSet;
use ui::Widget;

use glium::{self, Surface, glutin};
use glium::glutin::VirtualKeyCode;

pub struct GliumDisplay {
    display: glium::Display,
    events_loop: glium::glutin::EventsLoop,
    program: glium::Program,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

const VERTEX_SHADER_SRC: &'static str = r#"
  #version 140
  in vec2 position;
  in vec2 tex_coords;
  out vec2 v_tex_coords;
  void main() {
    v_tex_coords = tex_coords;
    gl_Position = vec4(position, 0.0, 1.0);
  }
"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
  #version 140
  in vec2 v_tex_coords;
  out vec4 color;
  uniform sampler2D tex;
  void main() {
    color = texture(tex, v_tex_coords);
  }
"#;

impl GliumDisplay {
    pub fn new() -> GliumDisplay {
        debug!("Initialize Glium Display adapter.");
        let events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(800, 600)
            .with_title("Hello world");
        let context = glium::glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC,
                                                  FRAGMENT_SHADER_SRC, None).unwrap();


        GliumDisplay {
            display,
            events_loop,
            program,
        }
    }
}

impl IO for GliumDisplay {
    fn process_input(&mut self, root: Rc<RefCell<Widget>>) {
        self.events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                InputAction::handle_action(process_window_event(event), Rc::clone(&root));
            }
        });
    }

    fn render_output(&mut self, _root: Ref<Widget>, _millis: u32) {
        let vertex1 = Vertex { position: [ 0.0,  0.0], tex_coords: [0.0, 0.9] };
        let vertex2 = Vertex { position: [ 0.0,  0.5], tex_coords: [0.0, 1.0] };
        let vertex3 = Vertex { position: [ 0.5,  0.0], tex_coords: [0.1, 0.9] };
        let vertex4 = Vertex { position: [ 0.5,  0.5], tex_coords: [0.1, 1.0] };
        let shape = vec![vertex1, vertex2, vertex3, vertex4];

        let image = ResourceSet::get_spritesheet("gui").unwrap().image.clone();
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::texture::Texture2d::new(&self.display, image).unwrap();

        let vertex_buffer = glium::VertexBuffer::new(&self.display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
        let uniforms = uniform! {
            tex: &texture,
        };

        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            smooth: Some(glium::draw_parameters::Smooth::Nicest),
            .. Default::default()
        };
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer, &indices, &self.program, &uniforms,
                    &params).unwrap();
        target.finish().unwrap();
    }
}

fn process_window_event(event: glutin::WindowEvent) -> Option<InputAction> {
    use glium::glutin::WindowEvent::*;
    match event {
        Closed => Some(InputAction::Exit),
        KeyboardInput { input, .. } => {
            CONFIG.get_input_action(process_keyboard_input(input))
        }
        _ => None,
    }
}

fn process_keyboard_input(input: glutin::KeyboardInput) -> Option<KeyboardEvent> {
    if input.state != glutin::ElementState::Pressed { return None; }
    trace!("Glium keyboard input {:?}", input);

    let key_code = match input.virtual_keycode {
        None => return None,
        Some(key) => key,
    };

    use io::keyboard_event::Key::*;
    use glium::glutin::VirtualKeyCode::*;
    let key = match key_code {
        A => KeyA,
        B => KeyB,
        C => KeyC,
        D => KeyD,
        E => KeyE,
        F => KeyF,
        G => KeyG,
        H => KeyH,
        I => KeyI,
        J => KeyJ,
        K => KeyK,
        L => KeyL,
        M => KeyM,
        N => KeyN,
        O => KeyO,
        P => KeyP,
        Q => KeyQ,
        R => KeyR,
        S => KeyS,
        T => KeyT,
        U => KeyU,
        V => KeyV,
        W => KeyW,
        X => KeyX,
        Y => KeyY,
        Z => KeyZ,
        VirtualKeyCode::Key0 => Key::Key0,
        VirtualKeyCode::Key1 => Key::Key1,
        VirtualKeyCode::Key2 => Key::Key2,
        VirtualKeyCode::Key3 => Key::Key3,
        VirtualKeyCode::Key4 => Key::Key4,
        VirtualKeyCode::Key5 => Key::Key5,
        VirtualKeyCode::Key6 => Key::Key6,
        VirtualKeyCode::Key7 => Key::Key7,
        VirtualKeyCode::Key8 => Key::Key8,
        VirtualKeyCode::Key9 => Key::Key9,
        Escape => KeyEscape,
        Back => KeyBackspace,
        Tab => KeyTab,
        Space => KeySpace,
        Return => KeyEnter,
        Grave => KeyGrave,
        Minus => KeyMinus,
        Equals => KeyEquals,
        LBracket => KeyLeftBracket,
        RBracket => KeyRightBracket,
        Semicolon => KeySemicolon,
        Apostrophe => KeySingleQuote,
        Comma => KeyComma,
        Period => KeyPeriod,
        Slash => KeySlash,
        Backslash => KeyBackslash,
        Home => KeyHome,
        End => KeyEnd,
        Insert => KeyInsert,
        Delete => KeyDelete,
        PageDown => KeyPageDown,
        PageUp => KeyPageUp,
        Up => KeyUp,
        Down => KeyDown,
        Left => KeyLeft,
        Right => KeyRight,
        F1 => KeyF1,
        F2 => KeyF2,
        F3 => KeyF3,
        F4 => KeyF4,
        F5 => KeyF5,
        F6 => KeyF6,
        F7 => KeyF7,
        F8 => KeyF8,
        F9 => KeyF9,
        F10 => KeyF10,
        F11 => KeyF11,
        F12 => KeyF12,
        _ => KeyUnknown,
    };

    Some(KeyboardEvent { key })
}
