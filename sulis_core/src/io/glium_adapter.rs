//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use crate::config::{Config, DisplayMode};
use crate::io::keyboard_event::Key;
use crate::io::*;
use crate::resource::ResourceSet;
use crate::ui::{Cursor, Widget};
use crate::util::{Point};

use glium::backend::Facade;
use glium::glutin::{
    ContextBuilder,
    Event, KeyboardInput, MouseButton, WindowEvent, VirtualKeyCode, ElementState, MouseScrollDelta,
    EventsLoop,
    MonitorId,
    WindowBuilder,
};
use glium::texture::{RawImage2d, SrgbTexture2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler};
use glium::{self, glutin, CapabilitiesSource, Rect, Surface};

const VERTEX_SHADER_SRC: &str = r#"
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

const FRAGMENT_SHADER_SRC: &str = r#"
  #version 140
  in vec2 v_tex_coords;
  out vec4 color;
  uniform sampler2D tex;
  uniform vec4 color_filter;
  uniform vec4 color_sec;

  void main() {
    color = color_filter * texture(tex, v_tex_coords) + color_sec;
  }
"#;

const SWAP_FRAGMENT_SHADER_SRC: &str = r#"
  #version 140
  in vec2 v_tex_coords;
  out vec4 color;
  uniform sampler2D tex;
  uniform vec4 color_filter;
  uniform bool color_swap_enabled;
  uniform float swap_hue;

  vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
  }

  vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
  }

  void main() {
    vec4 tex_color = texture(tex, v_tex_coords);

    vec3 hsv = rgb2hsv(tex_color.rgb);
    if (hsv.x < 0.9 && hsv.x > 0.8 ) {
      vec3 rgb = hsv2rgb(vec3(swap_hue, hsv.y, hsv.z));
      color = vec4(rgb.r, rgb.g, rgb.b, tex_color.a);
    } else {
      color = color_filter * tex_color;
    }
  }
"#;

pub struct GliumDisplay {
    display: glium::Display,
    events_loop: EventsLoop,
    base_program: glium::Program,
    swap_program: glium::Program,
    matrix: [[f32; 4]; 4],
    textures: HashMap<String, GliumTexture>,
    scale_factor: f64,
}

struct GliumTexture {
    texture: SrgbTexture2d,
    sampler_fn: Box<dyn Fn(Sampler<SrgbTexture2d>) -> Sampler<SrgbTexture2d>>,
}

pub struct GliumRenderer<'a> {
    target: &'a mut glium::Frame,
    display: &'a mut GliumDisplay,
    params: glium::DrawParameters<'a>,
    scale_factor: f64,
}

impl<'a> GliumRenderer<'a> {
    fn new(target: &'a mut glium::Frame, display: &'a mut GliumDisplay) -> GliumRenderer<'a> {
        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            clip_planes_bitmask: 0,

            ..Default::default()
        };

        let scale_factor = display.scale_factor;
        GliumRenderer {
            target,
            display,
            params,
            scale_factor,
        }
    }

    fn create_texture_if_missing(&mut self, texture_id: &str, draw_list: &DrawList) {
        if self.display.textures.get(texture_id).is_some() {
            return;
        }

        trace!(
            "Creating texture for ID '{}' of type '{:?}'",
            texture_id,
            draw_list.kind
        );
        let image = match draw_list.kind {
            DrawListKind::Sprite => ResourceSet::spritesheet(&texture_id).unwrap().image.clone(),
            DrawListKind::Font => ResourceSet::font(&texture_id).unwrap().image.clone(),
        };

        self.register_texture(
            texture_id,
            image,
            draw_list.texture_min_filter,
            draw_list.texture_mag_filter,
        );
    }
}

fn draw_to_surface<T: glium::Surface>(
    surface: &mut T,
    draw_list: DrawList,
    display: &GliumDisplay,
    params: &glium::DrawParameters,
) {
    let glium_texture = match display.textures.get(&draw_list.texture) {
        None => return,
        Some(texture) => texture,
    };

    let uniforms = uniform! {
        matrix: display.matrix,
        tex: (glium_texture.sampler_fn)(glium_texture.texture.sampled()),
        color_filter: draw_list.color_filter,
        color_sec: draw_list.color_sec,
        swap_hue: draw_list.swap_hue,
        scale: [
            [draw_list.scale[0], 0.0, 0.0, 0.0],
            [0.0, draw_list.scale[1], 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [draw_list.scale[0] - 1.0, 1.0 - draw_list.scale[1], 0.0, 1.0f32],
        ],
    };

    let vertex_buffer = glium::VertexBuffer::new(&display.display, &draw_list.quads).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let program = if draw_list.color_swap_enabled {
        &display.swap_program
    } else {
        &display.base_program
    };

    match surface.draw(&vertex_buffer, &indices, program, &uniforms, params) {
        Ok(()) => (),
        Err(e) => error!("Error drawing to surface: {:?}", e),
    }
}

impl<'a> GraphicsRenderer for GliumRenderer<'a> {
    fn set_scissor(&mut self, pos: Point, size: Size) {
        let window_size = match self.display.display.gl_window().window().get_inner_size() {
            None => {
                warn!("Unable to query display size.");
                return;
            }, Some(size) => size,
        };
        let (res_x, res_y) = Config::ui_size();
        let scale_x = self.scale_factor * window_size.width as f64 / res_x as f64;
        let scale_y = self.scale_factor * window_size.height as f64 / res_y as f64;

        let rect = glium::Rect {
            left: (pos.x as f64 * scale_x) as u32,
            bottom: ((res_y - (pos.y + size.height)) as f64 * scale_y) as u32,
            width: (size.width as f64 * scale_x) as u32,
            height: (size.height as f64 * scale_y) as u32,
        };
        self.params.scissor = Some(rect);
    }

    fn clear_scissor(&mut self) {
        self.params.scissor = None;
    }

    fn register_texture(
        &mut self,
        id: &str,
        image: ImageBuffer<Rgba<u8>, Vec<u8>>,
        min_filter: TextureMinFilter,
        mag_filter: TextureMagFilter,
    ) {
        let dims = image.dimensions();
        trace!("Registering texture '{}', {}x{}", id, dims.0, dims.1);
        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dims);
        let texture = SrgbTexture2d::new(&self.display.display, image).unwrap();

        let sampler_fn: Box<dyn Fn(Sampler<SrgbTexture2d>) -> Sampler<SrgbTexture2d>> =
            Box::new(move |sampler| {
                sampler
                    .magnify_filter(get_mag_filter(mag_filter))
                    .minify_filter(get_min_filter(min_filter))
            });

        self.display.textures.insert(
            id.to_string(),
            GliumTexture {
                texture,
                sampler_fn,
            },
        );
    }

    fn clear_texture_region(&mut self, id: &str, min_x: i32, min_y: i32, max_x: i32, max_y: i32) {
        let texture = self.display.textures.get(id).unwrap();
        let tex_height = texture.texture.height();
        let mut framebuffer =
            glium::framebuffer::SimpleFrameBuffer::new(&self.display.display, &texture.texture)
                .unwrap();

        let rect = Rect {
            left: min_x as u32,
            bottom: tex_height - max_y as u32,
            width: (max_x - min_x) as u32,
            height: (max_y - min_y) as u32,
        };

        framebuffer.clear(Some(&rect), Some((0.0, 0.0, 0.0, 0.0)), true, None, None);
    }

    fn clear_texture(&mut self, id: &str) {
        let texture = self.display.textures.get(id).unwrap();
        let mut framebuffer =
            glium::framebuffer::SimpleFrameBuffer::new(&self.display.display, &texture.texture)
                .unwrap();

        framebuffer.clear_color(0.0, 0.0, 0.0, 0.0);
    }

    fn has_texture(&self, id: &str) -> bool {
        self.display.textures.contains_key(id)
    }

    fn draw_to_texture(&mut self, texture_id: &str, draw_list: DrawList) {
        self.create_texture_if_missing(&draw_list.texture, &draw_list);
        let texture = self.display.textures.get(texture_id).unwrap();
        let mut framebuffer =
            glium::framebuffer::SimpleFrameBuffer::new(&self.display.display, &texture.texture)
                .unwrap();

        draw_to_surface(&mut framebuffer, draw_list, &self.display, &self.params);
    }

    fn draw(&mut self, draw_list: DrawList) {
        if draw_list.texture.is_empty() {
            return;
        }
        self.create_texture_if_missing(&draw_list.texture, &draw_list);

        draw_to_surface(self.target, draw_list, &self.display, &self.params);
    }
}

fn glium_error<E: ::std::fmt::Display>(e: E) -> Result<GliumDisplay, Error> {
    Err(Error::new(ErrorKind::Other, format!("{}", e)))
}

fn get_monitor(events_loop: &EventsLoop) -> MonitorId {
    let target_monitor = Config::monitor();
    for (index, monitor) in events_loop.get_available_monitors().enumerate() {
        if index == target_monitor {
            return monitor;
        }
    }
    warn!(
        "Unable to find a monitor with configured index {}",
        target_monitor
    );

    // return primary monitor
    events_loop.get_available_monitors().next().unwrap()
}

fn try_get_display(
    events_loop: &EventsLoop,
    monitor: MonitorId,
) -> Result<glium::Display, glium::backend::glutin::DisplayCreationError> {
    let (res_x, res_y) = Config::display_resolution();

    let (fullscreen, decorations) = match Config::display_mode() {
        DisplayMode::Window => (None, true),
        DisplayMode::BorderlessWindow => (None, false),
        DisplayMode::Fullscreen => (Some(monitor), false),
    };

    let dims = glutin::dpi::LogicalSize::new(res_x as f64, res_y as f64);

    let window = WindowBuilder::new()
        .with_dimensions(dims)
        .with_title("Sulis")
        .with_decorations(decorations)
        .with_fullscreen(fullscreen.clone());
    let context = ContextBuilder::new().with_pixel_format(24, 8);

    match glium::Display::new(window, context, &events_loop) {
        Ok(display) => return Ok(display),
        Err(e) => {
            warn!("Unable to create hardware accelerated display, falling back...");
            warn!("{}", e);
        }
    };

    let window = WindowBuilder::new()
        .with_dimensions(dims)
        .with_title("Sulis")
        .with_decorations(decorations)
        .with_fullscreen(fullscreen);
    let context = ContextBuilder::new()
        .with_hardware_acceleration(None)
        .with_pixel_format(24, 8);

    glium::Display::new(window, context, &events_loop)
}

impl GliumDisplay {
    pub fn new() -> Result<GliumDisplay, Error> {
        debug!("Initialize Glium Display adapter.");
        let events_loop = EventsLoop::new();
        let monitor = get_monitor(&events_loop);

        let display = match try_get_display(&events_loop, monitor.clone()) {
            Ok(display) => display,
            Err(e) => return glium_error(e),
        };

        let scale_factor = monitor.get_hidpi_factor();
        let physical_position = monitor.get_position();
        // TODO this doesn't work if you have monitors with different hidpi factors
        // would be very hard to solve in the general case, maybe just allow a configured
        // logical position
        let logical_position = physical_position.to_logical(scale_factor);
        display.gl_window().window().set_position(logical_position);

        info!("Initialized glium adapter:");
        info!("Version: {}", display.get_opengl_version_string());
        info!("Vendor: {}", display.get_opengl_vendor_string());
        info!("Renderer: {}", display.get_opengl_renderer_string());
        info!("Max viewport: {:?}", display.get_max_viewport_dimensions());
        info!(
            "Video memory available: {:?}",
            display.get_free_video_memory()
        );
        trace!("Extensions: {:#?}", display.get_context().get_extensions());
        trace!(
            "Capabilities: {:?}",
            display.get_context().get_capabilities()
        );

        info!("Using hi dpi scale factor: {}", scale_factor);

        let base_program = match glium::Program::from_source(
            &display,
            VERTEX_SHADER_SRC,
            FRAGMENT_SHADER_SRC,
            None,
        ) {
            Ok(prog) => prog,
            Err(e) => return glium_error(e),
        };

        let swap_program = match glium::Program::from_source(
            &display,
            VERTEX_SHADER_SRC,
            SWAP_FRAGMENT_SHADER_SRC,
            None,
        ) {
            Ok(prog) => prog,
            Err(e) => return glium_error(e),
        };

        display.gl_window().window().hide_cursor(true);

        let (ui_x, ui_y) = Config::ui_size();

        Ok(GliumDisplay {
            display,
            events_loop,
            base_program,
            swap_program,
            matrix: [
                [2.0 / ui_x as f32, 0.0, 0.0, 0.0],
                [0.0, 2.0 / ui_y as f32, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0, -1.0, 0.0, 1.0f32],
            ],
            textures: HashMap::new(),
            scale_factor,
        })
    }
}

const RESOLUTIONS: [(u32, u32); 8] = [
    (3840, 2160),
    (2560, 1440),
    (1920, 1080),
    (1768, 992),
    (1600, 900),
    (1536, 864),
    (1366, 768),
    (1280, 720),
];

impl IO for GliumDisplay {
    fn get_display_configurations(&self) -> Vec<DisplayConfiguration> {
        let mut configs = Vec::new();

        for (index, monitor_id) in self.events_loop.get_available_monitors().enumerate() {
            // glium get_name() does not seem to return anything useful in most cases
            let name = format!("Monitor {}", index + 1);

            let dims: (u32, u32) = monitor_id
                .get_dimensions()
                .to_logical(monitor_id.get_hidpi_factor())
                .into();

            let mut resolutions = Vec::new();
            for (w, h) in RESOLUTIONS.iter() {
                let (w, h) = (*w, *h);

                if w > dims.0 || h > dims.1 { continue; }

                let monitor_size = w == dims.0 && h == dims.1;

                resolutions.push(Resolution { width: w, height: h, fullscreen: true, monitor_size });
            }

            configs.push(DisplayConfiguration {
                name,
                index,
                resolutions,
            });
        }

        configs
    }

    fn process_input(&mut self, root: Rc<RefCell<Widget>>) {
        let (ui_x, ui_y) = Config::ui_size();
        let mut mouse_move: Option<(f32, f32)> = None;
        let display_size = match self.display.gl_window().window().get_inner_size() {
            None => {
                crate::util::error_and_exit("Unable to query display size.");
                unreachable!();
            }, Some(size) => size,
        };

        self.events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        let mouse_x = (ui_x as f64 * position.x / display_size.width) as f32;
                        let mouse_y = (ui_y as f64 * position.y / display_size.height) as f32;
                        mouse_move = Some((mouse_x, mouse_y));
                    }
                    _ => {
                        for action in process_window_event(event) {
                            InputAction::handle_action(action, &root);
                        }
                    }
                }
            }
        });

        // merge all mouse move events into at most one per frame
        if let Some((mouse_x, mouse_y)) = mouse_move {
            InputAction::handle_action(InputAction::MouseMove(mouse_x, mouse_y), &root);
        }
    }

    fn render_output(&mut self, root: Ref<Widget>, millis: u32) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        {
            let scale_factor = self.scale_factor;
            let mut renderer = GliumRenderer::new(&mut target, self);
            let (width, height) = renderer.target.get_dimensions();
            let width = (width as f64 / scale_factor).round() as i32;
            let height = (height as f64 / scale_factor).round() as i32;
            let pixel_size = Point::new(width, height);
            root.draw(&mut renderer, pixel_size, millis);

            Cursor::draw(&mut renderer, millis);
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

fn process_window_event(event: WindowEvent) -> Vec<InputAction> {
    use WindowEvent::*;
    match event {
        CloseRequested => vec![InputAction::Exit],
        ReceivedCharacter(c) => vec![InputAction::CharReceived(c)],
        KeyboardInput { input, .. } => {
            let mut result = Vec::new();
            let kb_event = match process_keyboard_input(input) {
                None => return Vec::new(),
                Some(evt) => evt,
            };
            result.push(InputAction::RawKey(kb_event.key));
            match Config::get_input_action(kb_event) {
                None => (),
                Some(action) => result.push(action),
            };
            result
        }
        MouseInput { state, button, .. } => {
            use crate::config::RawClick;
            let kind = match button {
                MouseButton::Left => Config::get_click_action(RawClick::Left),
                MouseButton::Right => Config::get_click_action(RawClick::Right),
                MouseButton::Middle => Config::get_click_action(RawClick::Middle),
                _ => return Vec::new(),
            };

            match state {
                ElementState::Pressed => vec![InputAction::MouseDown(kind)],
                ElementState::Released => vec![InputAction::MouseUp(kind)],
            }
        }
        MouseWheel { delta, .. } => {
            let amount = match delta {
                MouseScrollDelta::LineDelta(_, y) => y,
                MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
            };

            // scrolling the mouse wheeel seems to be buggy at the moment, only take some events
            let amount = if (amount - 1.0).abs() < std::f32::EPSILON {
                1
            } else if (amount + 1.0).abs() < std::f32::EPSILON {
                -1
            } else {
                return Vec::new();
            };

            vec![InputAction::MouseScroll(amount)]
        }
        _ => Vec::new(),
    }
}

fn process_keyboard_input(input: KeyboardInput) -> Option<KeyboardEvent> {
    if input.state != ElementState::Pressed {
        return None;
    }
    trace!("Glium keyboard input {:?}", input);

    let key_code = match input.virtual_keycode {
        None => return None,
        Some(key) => key,
    };

    use crate::io::keyboard_event::Key::*;
    use VirtualKeyCode::*;
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
        Minus | Subtract => KeyMinus,
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
