use std::cell::{Ref, RefCell};
use std::rc::Rc;

use config::CONFIG;
use io::{InputAction, KeyboardEvent, IO};
use io::keyboard_event::Key;
use resource::ResourceSet;
use ui::Widget;

use glium::{self, Surface, glutin};
use glium::glutin::VirtualKeyCode;

const VERTEX_SHADER_SRC: &'static str = r#"
  #version 140
  in vec2 position;
  in vec2 tex_coords;
  out vec2 v_tex_coords;
  uniform mat4 matrix;
  void main() {
    v_tex_coords = tex_coords;
    gl_Position = matrix * vec4(position, 0.0, 1.0);
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

pub struct GliumDisplay<'a> {
    display: glium::Display,
    events_loop: glium::glutin::EventsLoop,
    program: glium::Program,
    params: glium::DrawParameters<'a>,
    texture: glium::texture::SrgbTexture2d,
    font_tex: glium::texture::SrgbTexture2d,

    matrix: [[f32; 4]; 4],
}

impl<'a> GliumDisplay<'a> {
    pub fn new() -> GliumDisplay<'a> {
        debug!("Initialize Glium Display adapter.");
        let events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(CONFIG.display.width_pixels, CONFIG.display.height_pixels)
            .with_title("Rust Game");
        let context = glium::glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC,
                                                  FRAGMENT_SHADER_SRC, None).unwrap();

        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            .. Default::default()
        };

        let image = ResourceSet::get_spritesheet("gui").unwrap().image.clone();
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

        let font_image = ResourceSet::get_font("large").unwrap().image.clone();
        let font_image_dim = font_image.dimensions();
        let font_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&font_image.into_raw(), font_image_dim);
        let font_tex = glium::texture::SrgbTexture2d::new(&display, font_image).unwrap();

        GliumDisplay {
            display,
            events_loop,
            program,
            params,
            texture,
            font_tex,
            matrix: [
                [2.0 / CONFIG.display.width as f32, 0.0, 0.0, 0.0],
                [0.0, 2.0 / CONFIG.display.height as f32, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0 , -1.0, 0.0, 1.0f32],
            ],
        }
    }

    fn draw_widget(&self, widget: Ref<Widget>, target: &mut glium::Frame) {
        if let Some(ref image) = widget.state.background {
            let uniforms = uniform! {
                matrix: self.matrix,
                tex: self.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            };

            let pos = &widget.state.position;
            let size = &widget.state.size;
            let quads = image.get_quads(&widget.state.animation_state, pos, size);

            for quad in quads {
                let vertex_buffer = glium::VertexBuffer::new(&self.display, &quad.vertices).unwrap();
                let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
                target.draw(&vertex_buffer, &indices, &self.program, &uniforms, &self.params).unwrap();
            }
        }

        let uniforms = uniform! {
            matrix: self.matrix,
            tex: self.font_tex.sampled().minify_filter(glium::uniforms::MinifySamplerFilter::Linear),
        };
        for quad in widget.kind.get_quads(&widget, 0) {
            let vertex_buffer = glium::VertexBuffer::new(&self.display, &quad.vertices).unwrap();
            let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
            target.draw(&vertex_buffer, &indices, &self.program, &uniforms, &self.params).unwrap();
        }

        for child in widget.children.iter() {
            self.draw_widget(child.borrow(), target);
        }
    }
}

impl<'a> IO for GliumDisplay<'a> {
    fn process_input(&mut self, root: Rc<RefCell<Widget>>) {
        self.events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                InputAction::handle_action(process_window_event(event), Rc::clone(&root));
            }
        });
    }

    fn render_output(&mut self, root: Ref<Widget>, _millis: u32) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        self.draw_widget(root, &mut target);
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
