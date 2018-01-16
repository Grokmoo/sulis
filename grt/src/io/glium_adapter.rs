use std::collections::HashMap;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

use config::CONFIG;
use io::*;
use io::keyboard_event::Key;
use io::event::ClickKind;
use resource::ResourceSet;
use ui::Widget;
use util::Point;

use glium::{self, Surface, glutin};
use glium::glutin::VirtualKeyCode;
use glium::texture::{RawImage2d, SrgbTexture2d};
use glium::uniforms::{MinifySamplerFilter, MagnifySamplerFilter, Sampler};

const VERTEX_SHADER_SRC: &'static str = r#"
  #version 140
  in vec2 position;
  in vec2 tex_coords;
  out vec2 v_tex_coords;
  uniform mat4 matrix;
  uniform mat4 scale;
  void main() {
    v_tex_coords = tex_coords;
    gl_Position = scale * matrix * vec4(position, 0.0, 1.0);
  }
"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
  #version 140
  in vec2 v_tex_coords;
  out vec4 color;
  uniform sampler2D tex;
  uniform vec4 color_filter;
  void main() {
    color = color_filter * texture(tex, v_tex_coords);
  }
"#;

pub struct GliumDisplay {
    display: glium::Display,
    events_loop: glium::glutin::EventsLoop,
    program: glium::Program,
    matrix: [[f32; 4]; 4],
    textures: HashMap<String, GliumTexture>,
}

struct GliumTexture {
    texture: SrgbTexture2d,
    sampler_fn: Box<Fn(Sampler<SrgbTexture2d>) -> Sampler<SrgbTexture2d>>,
}

pub struct GliumRenderer<'a> {
    target: &'a mut glium::Frame,
    display: &'a mut GliumDisplay,
    params: glium::DrawParameters<'a>,
}

impl<'a> GliumRenderer<'a> {
    fn new(target: &'a mut glium::Frame, display: &'a mut GliumDisplay) -> GliumRenderer<'a> {
        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            .. Default::default()
        };

        GliumRenderer {
            target,
            display,
            params,
        }
    }

    fn create_texture_if_missing(&mut self, texture_id: &str, draw_list: &DrawList) {
        if self.display.textures.get(texture_id).is_some() {
            return;
        }

        trace!("Creating texture for ID '{}' of type '{:?}'", texture_id, draw_list.kind);
        let image = match draw_list.kind {
            DrawListKind::Sprite => ResourceSet::get_spritesheet(&texture_id)
                .unwrap().image.clone(),
            DrawListKind::Font => ResourceSet::get_font(&texture_id)
                .unwrap().image.clone(),
        };

        self.register_texture(texture_id, image, draw_list.texture_min_filter, draw_list.texture_mag_filter);
    }

    fn draw_widget_tree(&mut self, widget: Ref<Widget>, millis: u32) {
        let pixel_size = Point::from_tuple(self.target.get_dimensions());

        if let Some(ref image) = widget.state.background {
            let x = widget.state.position.x as f32;
            let y = widget.state.position.y as f32;
            let w = widget.state.size.width as f32;
            let h = widget.state.size.height as f32;
            image.draw_graphics_mode(self, &widget.state.animation_state, x, y, w, h);
        }

        widget.kind.draw_graphics_mode(self, pixel_size, &widget, millis);

        for child in widget.children.iter() {
            self.draw_widget_tree(child.borrow(), millis);
        }
    }
}

impl<'a> GraphicsRenderer for GliumRenderer<'a> {
    fn register_texture(&mut self, id: &str, image: ImageBuffer<Rgba<u8>, Vec<u8>>,
                        min_filter: TextureMinFilter, mag_filter: TextureMagFilter) {

        let dims = image.dimensions();
        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dims);
        let texture = SrgbTexture2d::new(&self.display.display, image).unwrap();

        let sampler_fn: Box<Fn(Sampler<SrgbTexture2d>) -> Sampler<SrgbTexture2d>> =
            Box::new(move |sampler| {
                sampler.magnify_filter(get_mag_filter(mag_filter))
                    .minify_filter(get_min_filter(min_filter))
            });

        self.display.textures.insert(id.to_string(), GliumTexture { texture, sampler_fn });
    }

    fn deregister_texture(&mut self, id: &str) {
        self.display.textures.remove(id);
    }

    fn draw(&mut self, draw_list: DrawList) {
        self.create_texture_if_missing(&draw_list.texture, &draw_list);

        let glium_texture = match self.display.textures.get(&draw_list.texture) {
            None => return,
            Some(texture) => texture,
        };

        let uniforms = uniform! {
            matrix: self.display.matrix,
            tex: (glium_texture.sampler_fn)(glium_texture.texture.sampled()),
            color_filter: draw_list.color_filter,
            scale: [
                [draw_list.scale[0], 0.0, 0.0, 0.0],
                [0.0, draw_list.scale[1], 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [draw_list.scale[0] - 1.0, 1.0 - draw_list.scale[1], 0.0, 1.0f32],
            ],
        };

        for quad in draw_list.quads {
            let vertex_buffer = glium::VertexBuffer::new(&self.display.display, &quad).unwrap();
            let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
            self.target.draw(&vertex_buffer, &indices, &self.display.program,
                        &uniforms, &self.params).unwrap();
        }
    }
}

impl GliumDisplay {
    pub fn new() -> GliumDisplay {
        debug!("Initialize Glium Display adapter.");
        let events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(CONFIG.display.width_pixels, CONFIG.display.height_pixels)
            .with_title("Rust Game");
        let context = glium::glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC,
                                                  FRAGMENT_SHADER_SRC, None).unwrap();

        GliumDisplay {
            display,
            events_loop,
            program,
            matrix: [
                [2.0 / CONFIG.display.width as f32, 0.0, 0.0, 0.0],
                [0.0, 2.0 / CONFIG.display.height as f32, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0 , -1.0, 0.0, 1.0f32],
            ],
            textures: HashMap::new(),
        }
    }
}

impl IO for GliumDisplay {
    fn process_input(&mut self, root: Rc<RefCell<Widget>>) {
        let display_size = self.display.gl_window().get_inner_size();
        self.events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                InputAction::handle_action(process_window_event(event, display_size),
                    Rc::clone(&root));
            }
        });
    }

    fn render_output(&mut self, root: Ref<Widget>, millis: u32) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        {
            let mut renderer = GliumRenderer::new(&mut target, self);
            renderer.draw_widget_tree(root, millis);
        }
        target.finish().unwrap();
    }
}

fn get_mag_filter(filter: TextureMagFilter) -> MagnifySamplerFilter {
    match filter {
        TextureMagFilter::Nearest => MagnifySamplerFilter::Nearest,
        TextureMagFilter::Linear => MagnifySamplerFilter::Linear,
    }
}

fn get_min_filter(filter: TextureMinFilter) -> MinifySamplerFilter {
    match filter {
        TextureMinFilter::Nearest => MinifySamplerFilter::Nearest,
        TextureMinFilter::Linear => MinifySamplerFilter::Linear,
        TextureMinFilter::NearestMipmapNearest => MinifySamplerFilter::NearestMipmapNearest,
        TextureMinFilter::LinearMipmapNearest => MinifySamplerFilter::LinearMipmapNearest,
        TextureMinFilter::NearestMipmapLinear => MinifySamplerFilter::NearestMipmapLinear,
        TextureMinFilter::LinearMipmapLinear => MinifySamplerFilter::LinearMipmapLinear,
    }
}

fn process_window_event(event: glutin::WindowEvent,
                        display_size: Option<(u32, u32)>) -> Option<InputAction> {
    use glium::glutin::WindowEvent::*;
    match event {
        Closed => Some(InputAction::Exit),
        KeyboardInput { input, .. } => CONFIG.get_input_action(process_keyboard_input(input)),
        CursorMoved { position, .. } => {
            let (width, height) = display_size.unwrap();
            let mouse_x = (CONFIG.display.width as f64 * position.0 / width as f64) as f32;
            let mouse_y = (CONFIG.display.height as f64 * position.1 / height as f64) as f32;

            Some(InputAction::MouseMove(mouse_x, mouse_y))
        },
        MouseInput { state, button, .. } => {
            let kind = match button {
                glium::glutin::MouseButton::Left => ClickKind::Left,
                glium::glutin::MouseButton::Right => ClickKind::Right,
                glium::glutin::MouseButton::Middle => ClickKind::Middle,
                _ => return None,
            };

            match state {
                glium::glutin::ElementState::Pressed => Some(InputAction::MouseDown(kind)),
                glium::glutin::ElementState::Released => Some(InputAction::MouseUp(kind)),
            }
        },
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
