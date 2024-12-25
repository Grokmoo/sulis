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

use std::num::NonZeroU32;
use std::time;
use std::collections::HashMap;
use std::error::Error;

use crate::config::{Config, DisplayMode};
use crate::io::keyboard_event::Key;
use crate::{io::*, util};
use crate::resource::ResourceSet;
use crate::ui::{Cursor, Widget};
use crate::util::{Point, get_elapsed_millis};

use glium::glutin::context::ContextAttributesBuilder;
use glium::glutin::display::GetGlDisplay;
use glium::glutin::prelude::{GlDisplay, NotCurrentGlContext};
use glium::winit::application::ApplicationHandler;
use glium::winit::raw_window_handle::{HandleError, HasWindowHandle};
use glium::{CapabilitiesSource, Display};
use glium::backend::Facade;
use glium::backend::glutin::simple_window_builder::GliumEventLoop;
use glium::glutin::config::{ColorBufferType, ConfigTemplateBuilder};
use glium::glutin::surface::{GlSurface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glium::texture::{RawImage2d, Texture2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler};
use glium::winit::dpi::{LogicalPosition, LogicalSize};
use glium::winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use glium::winit::monitor::MonitorHandle;
use glium::winit::window::{Fullscreen, Window, WindowId};
use glium::winit::keyboard::{KeyCode, PhysicalKey};
use glium::winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, StartCause, WindowEvent};
use glium::{self, Rect, Surface};
use glutin_winit::DisplayBuilder;

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
    display: glium::Display<WindowSurface>,
    window: Window,
    monitor: MonitorHandle,
    base_program: glium::Program,
    swap_program: glium::Program,
    matrix: [[f32; 4]; 4],
    textures: HashMap<String, GliumTexture>,
    scale_factor: f64,
}

struct GliumTexture {
    texture: Texture2d,
    sampler_fn: Box<dyn Fn(Sampler<Texture2d>) -> Sampler<Texture2d>>,
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
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            clip_planes_bitmask: 0,

            ..Default::default()
        };

        GliumRenderer {
            target,
            display,
            params,
        }
    }

    fn create_texture_if_missing(&mut self, texture_id: &str, draw_list: &DrawList) {
        if self.display.textures.contains_key(texture_id) {
            return;
        }

        trace!(
            "Creating texture for ID '{}' of type '{:?}'",
            texture_id,
            draw_list.kind
        );
        let image = match draw_list.kind {
            DrawListKind::Sprite => ResourceSet::spritesheet(texture_id).unwrap().image.clone(),
            DrawListKind::Font => ResourceSet::font(texture_id).unwrap().image.clone(),
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

    match surface.draw(&vertex_buffer, indices, program, &uniforms, params) {
        Ok(()) => (),
        Err(e) => error!("Error drawing to surface: {:?}", e),
    }
}

impl GraphicsRenderer for GliumRenderer<'_> {
    fn set_scissor(&mut self, pos: Point, size: Size) {
        let window_size = self.display.window.inner_size();
        let (res_x, res_y) = Config::ui_size();
        let scale_x = window_size.width as f64 / res_x as f64;
        let scale_y = window_size.height as f64 / res_y as f64;

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
        let texture = Texture2d::new(&self.display.display, image).unwrap();

        let sampler_fn: Box<dyn Fn(Sampler<Texture2d>) -> Sampler<Texture2d>> =
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

        draw_to_surface(&mut framebuffer, draw_list, self.display, &self.params);
    }

    fn draw(&mut self, draw_list: DrawList) {
        if draw_list.texture.is_empty() {
            return;
        }
        self.create_texture_if_missing(&draw_list.texture, &draw_list);

        draw_to_surface(self.target, draw_list, self.display, &self.params);
    }
}

fn configured_fullscreen(res_x: u32, res_y: u32, monitor: MonitorHandle) -> (Option<Fullscreen>, bool) {
    match Config::display_mode() {
        DisplayMode::Window => (None, true),
        DisplayMode::BorderlessWindow => (Some(Fullscreen::Borderless(Some(monitor.clone()))), false),
        DisplayMode::Fullscreen => {
            let mut selected_mode = None;
            for mode in monitor.video_modes() {
                let (mx, my): (u32, u32) = mode.size().into();

                if mx == res_x && my == res_y {
                    selected_mode = Some(mode);
                    break;
                }
            }
            match selected_mode {
                None => {
                    warn!("Unable to find a fullscreen video mode matching {} by {}", res_x, res_y);
                    warn!("Falling back to windowed mode.");
                    (None, false)
                },
                Some(mode) => (Some(Fullscreen::Exclusive(mode)), false),
            }
        }
    }
}

fn get_monitor(window: &Window) -> MonitorHandle {
    let target_monitor = Config::monitor();
    for (index, monitor) in window.available_monitors().enumerate() {
        if index == target_monitor {
            return monitor;
        }
    }
    warn!(
        "Unable to find a monitor with configured index {}",
        target_monitor
    );
    window.primary_monitor().unwrap()
}

#[derive(Debug)]
pub enum DisplayCreationError {
    CreateWindow,
    BuildDisplayError(Box<dyn Error>),
    WindowHandle(HandleError),
    WindowSurface(glium::glutin::error::Error),
    GlContext(glium::glutin::error::Error),
    GlContextCurrent(glium::glutin::error::Error),
    GlVersion(glium::IncompatibleOpenGl),
    SetVsync(glium::glutin::error::Error),
}

impl std::fmt::Display for DisplayCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DisplayCreationError::*;
        match self {
            CreateWindow => write!(f, "Unable to create display"),
            BuildDisplayError(e) => write!(f, "Build display error: {e}"),
            WindowHandle(e) => write!(f, "Unable to get window handle {e}"),
            WindowSurface(e) => write!(f, "Unable to setup window surface {e}"),
            GlContext(e) => write!(f, "Unable to create GL context {e}"),
            GlContextCurrent(e) => write!(f, "Unable to make GL context current {e}"),
            GlVersion(e) => write!(f, "GL Version incompatible: {e}"),
            SetVsync(e) => write!(f, "Unable to set vsync mode: {e}"),
        }
    }
}

impl Error for DisplayCreationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use DisplayCreationError::*;
        match self {
            CreateWindow => None,
            BuildDisplayError(e) => Some(e.as_ref()),
            WindowHandle(e) => Some(e),
            WindowSurface(e) => Some(e),
            GlContext(e) => Some(e),
            GlContextCurrent(e) => Some(e),
            GlVersion(e) => Some(e),
            SetVsync(e) => Some(e),
        }
    }
}

fn try_get_display(
    event_loop: &EventLoop<()>,
    hardware_acceleration: bool,
) -> Result<(glium::Display<WindowSurface>, Window), DisplayCreationError> {
    let (res_x, res_y) = Config::display_resolution();
    let vsync = Config::vsync_enabled();

    let dims = LogicalSize::new(res_x as f64, res_y as f64);

    let attrs = Window::default_attributes()
        .with_title("Sulis")
        .with_inner_size(dims);

    let hardware_accelerated = if hardware_acceleration { Some(true) } else { None };

    let config_template_builder = ConfigTemplateBuilder::new()
        .with_depth_size(24)
        .prefer_hardware_accelerated(hardware_accelerated)
        .with_alpha_size(8)
        .with_buffer_type(ColorBufferType::Rgb { r_size: 8, g_size: 8, b_size: 8 });

    let display_builder = DisplayBuilder::new().with_window_attributes(Some(attrs));

    let (window, gl_config) = event_loop.build(display_builder, config_template_builder, |mut configs| {
        configs.next().unwrap()
    }).map_err(DisplayCreationError::BuildDisplayError)?;

    let window = match window {
        None => return Err(DisplayCreationError::CreateWindow),
        Some(window) => window,
    };

    let handle = window.window_handle().map_err(DisplayCreationError::WindowHandle)?;

    let (w, h): (u32, u32) = window.inner_size().into();

    let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new()
        .build(handle.into(), NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap());

    let surface = unsafe {
        gl_config.display().create_window_surface(&gl_config, &surface_attrs).map_err(DisplayCreationError::WindowSurface)?
    };

    let swap_interval = if vsync {
        SwapInterval::Wait(NonZeroU32::new(1).unwrap())
    } else {
        SwapInterval::DontWait
    };

    let context_attributes = ContextAttributesBuilder::new().build(Some(handle.into()));

    let context = unsafe {
        gl_config.display().create_context(&gl_config, &context_attributes).map_err(DisplayCreationError::GlContext)?
    };

    let current_context = context.make_current(&surface).map_err(DisplayCreationError::GlContextCurrent)?;

    surface.set_swap_interval(&current_context, swap_interval).map_err(DisplayCreationError::SetVsync)?;

    let display = Display::from_context_surface(current_context, surface).map_err(DisplayCreationError::GlVersion)?;

    Ok((display, window))
}

fn configure_window(window: &Window, monitor: MonitorHandle) -> f64 {
    let (res_x, res_y) = Config::display_resolution();

    let (fullscreen, decorations) = configured_fullscreen(res_x, res_y, monitor.clone());

    window.set_fullscreen(fullscreen);
    window.set_decorations(decorations);

    let scale_factor = monitor.scale_factor();
    let physical_position = monitor.position();
    // TODO this doesn't work if you have monitors with different hidpi factors
    // would be very hard to solve in the general case, maybe just allow a configured
    // logical position
    let logical_position: LogicalPosition<f64> = physical_position.to_logical(scale_factor);
    window.set_outer_position(logical_position);

    scale_factor
}

pub struct GliumSystem {
    pub io: GliumDisplay,
    pub event_loop: EventLoop<()>,
    pub audio: Option<AudioDevice>,
}

pub fn create_system() -> Result<GliumSystem, Box<dyn Error>> {
    let (io, event_loop) = glium_adapter::GliumDisplay::new()?;
    let audio = create_audio_device();

    Ok(GliumSystem { io, event_loop, audio })
}

impl GliumDisplay {
    pub fn new() -> Result<(GliumDisplay, EventLoop<()>), Box<dyn Error>> {
        debug!("Initialize Glium Display adapter.");
        let event_loop = EventLoop::new()?;

        let (display, window) = match try_get_display(&event_loop, true) {
            Err(e) => {
                log::warn!("Error creating hardware accelerated display: {e}");
                log::warn!("Falling back to software display");
                try_get_display(&event_loop, false)?
            }, Ok(val) => val,
        };

        let monitor = get_monitor(&window);

        let scale_factor = configure_window(&window, monitor.clone());

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

        let base_program = glium::Program::from_source(
            &display,
            VERTEX_SHADER_SRC,
            FRAGMENT_SHADER_SRC,
            None,
        )?;

        let swap_program = glium::Program::from_source(
            &display,
            VERTEX_SHADER_SRC,
            SWAP_FRAGMENT_SHADER_SRC,
            None,
        )?;

        window.set_cursor_visible(false);

        let (ui_x, ui_y) = Config::ui_size();

        Ok((GliumDisplay {
            display,
            window,
            monitor,
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
        }, event_loop))
    }

    pub(crate) fn get_display_configurations(&self) -> Vec<DisplayConfiguration> {
        let mut configs = Vec::new();

        for (index, monitor_id) in self.window.available_monitors().enumerate() {
            // glium get_name() does not seem to return anything useful in most cases
            let name = format!("Monitor {}", index + 1);

            let dims: (u32, u32) = monitor_id.size().into();

            let mut resolutions = Vec::new();
            for (w, h) in RESOLUTIONS.iter() {
                let (w, h) = (*w, *h);

                if w > dims.0 || h > dims.1 { continue; }

                let fullscreen = monitor_id.video_modes().any(|mode| mode.size().width == w && mode.size().height == h);
                let monitor_size = w == dims.0 && h == dims.1;

                resolutions.push(Resolution { width: w, height: h, fullscreen, monitor_size });
            }

            configs.push(DisplayConfiguration {
                name,
                index,
                resolutions,
            });
        }

        configs
    }

    fn render_output(&mut self, root: &Widget, millis: u32) {
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

pub(crate) fn main_loop(system: GliumSystem, updater: Box<dyn ControlFlowUpdater>) {
    let io = system.io;
    let event_loop = system.event_loop;
    let audio = system.audio;
    let root = updater.root();

    let scale = io.scale_factor;
    let (ui_x, ui_y) = Config::ui_size();
    let mouse_move: Option<(f32, f32)> = None;
    let display_size: LogicalSize<f64> = io.window.inner_size().to_logical(scale);

    let frame_time = time::Duration::from_secs_f32(1.0 / Config::frame_rate() as f32);

    info!("Starting main loop.");
    let main_loop_start_time = time::Instant::now();

    let frames = 0;
    let render_time = time::Duration::from_secs(0);
    let last_start_time = time::Instant::now();

    let last_elapsed = 0;
    let total_elapsed = 0;

    let mut app = GliumApp {
        io, audio, updater, display_size, scale, root, ui_x, ui_y, mouse_move, frame_time,
        frames, render_time, last_start_time, main_loop_start_time, last_elapsed, total_elapsed,
    };

    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("{e}");
        util::error_and_exit("There was a fatal error initializing the display system.");
    }
}

struct GliumApp {
    io: GliumDisplay,
    audio: Option<AudioDevice>,
    updater: Box<dyn ControlFlowUpdater>,

    display_size: LogicalSize<f64>,
    scale: f64,
    root: Rc<RefCell<Widget>>,
    ui_x: i32,
    ui_y: i32,
    mouse_move: Option<(f32, f32)>,

    frames: u32,
    frame_time: std::time::Duration,
    render_time: std::time::Duration,
    last_start_time: std::time::Instant,
    main_loop_start_time: std::time::Instant,
    last_elapsed: u32,
    total_elapsed: u32,
}

impl ApplicationHandler for GliumApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) { }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if Config::vsync_enabled() {
            self.io.window.request_redraw();
        }

        if let StartCause::ResumeTimeReached { .. } = cause {
            self.io.window.request_redraw();
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        let secs = self.render_time.as_secs() as f64 + self.render_time.subsec_nanos() as f64 * 1e-9;
        log::info!("Runtime: {:.2} seconds", (time::Instant::now() - self.main_loop_start_time).as_secs_f32());
        info!(
            "Rendered {} frames with total render time {:.4} seconds",
            self.frames, secs
        );
        info!(
            "Average frame render time: {:.2} milliseconds",
            1000.0 * secs / self.frames as f64
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                self.display_size = size.to_logical(self.scale);
            },
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale = scale_factor;
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                self.last_elapsed = get_elapsed_millis(self.last_start_time.elapsed());
                self.last_start_time = time::Instant::now();
                self.total_elapsed = get_elapsed_millis(self.main_loop_start_time.elapsed());

                // merge all mouse move events into at most one per frame
                if let Some((mouse_x, mouse_y)) = self.mouse_move {
                    InputAction::mouse_move(mouse_x, mouse_y).handle(&self.root);
                }
                self.mouse_move = None;

                self.root = self.updater.update(self.last_elapsed);
                if self.updater.is_exit() {
                    event_loop.exit();
                } else if self.updater.recreate_window() {
                    let window = &self.io.window;
                    let (res_x, res_y) = Config::display_resolution();
                    let (fullscreen, decorations) = configured_fullscreen(res_x, res_y, self.io.monitor.clone());
                    let set_res = fullscreen.is_none();

                    window.set_fullscreen(fullscreen);
                    window.set_decorations(decorations);

                    if set_res {
                        let dims = LogicalSize::new(res_x as f64, res_y as f64);
                        let _ = window.request_inner_size(dims);
                    }

                    self.audio = create_audio_device();
                }

                Audio::update(self.audio.as_mut(), self.last_elapsed);

                self.io.render_output(&self.root.borrow(), self.total_elapsed);

                self.render_time += self.last_start_time.elapsed();
                self.frames += 1;
                
                if !Config::vsync_enabled() {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(self.last_start_time + self.frame_time));
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let mouse_x = (self.ui_x as f64 * position.x / self.display_size.width) as f32 / self.scale as f32;
                let mouse_y = (self.ui_y as f64 * position.y / self.display_size.height) as f32 / self.scale as f32;
                self.mouse_move = Some((mouse_x, mouse_y));
            },
            event => {
                for action in process_window_event(event) {
                    action.handle(&self.root);
                }
            }
        }
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
        CloseRequested => vec![InputAction::exit()],
        KeyboardInput { event, .. } => {
            let mut result = Vec::new();
            let kb_event = match process_keyboard_input(&event) {
                None => return Vec::new(),
                Some(evt) => evt,
            };

            if matches!(kb_event.state, InputActionState::Started) {
                result.push(InputAction::raw_key(kb_event.key));
            }
            
            if let Some(action) = Config::get_input_action(kb_event) {
                result.push(action);
            }

            if let Some(text) = event.text.as_ref() {
                if event.state == ElementState::Pressed {
                    for c in text.chars() {
                        result.push(InputAction::char_received(c));
                    }
                }
            }

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
                ElementState::Pressed => vec![InputAction::mouse_pressed(kind)],
                ElementState::Released => vec![InputAction::mouse_released(kind)],
            }
        }
        MouseWheel { delta, .. } => {
            let amount = match delta {
                MouseScrollDelta::LineDelta(_, y) => y,
                MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
            };

            // scrolling the mouse wheeel seems to be buggy at the moment, only take some events
            let amount = if (amount - 1.0).abs() < f32::EPSILON {
                1
            } else if (amount + 1.0).abs() < f32::EPSILON {
                -1
            } else {
                return Vec::new();
            };

            vec![InputAction::mouse_scroll(amount)]
        }
        _ => Vec::new(),
    }
}

fn process_keyboard_input(event: &KeyEvent) -> Option<KeyboardEvent> {
    trace!("Winit keyboard input {:?}", event);

    let key_code = match event.physical_key {
        PhysicalKey::Code(key_code) => key_code,
        PhysicalKey::Unidentified(_) => return None,
    };

    use KeyCode::*;
    let key = match key_code {
        KeyA => Key::KeyA,
        KeyB => Key::KeyB,
        KeyC => Key::KeyC,
        KeyD => Key::KeyD,
        KeyE => Key::KeyE,
        KeyF => Key::KeyF,
        KeyG => Key::KeyG,
        KeyH => Key::KeyH,
        KeyI => Key::KeyI,
        KeyJ => Key::KeyJ,
        KeyK => Key::KeyK,
        KeyL => Key::KeyL,
        KeyM => Key::KeyM,
        KeyN => Key::KeyN,
        KeyO => Key::KeyO,
        KeyP => Key::KeyP,
        KeyQ => Key::KeyQ,
        KeyR => Key::KeyR,
        KeyS => Key::KeyS,
        KeyT => Key::KeyT,
        KeyU => Key::KeyU,
        KeyV => Key::KeyV,
        KeyW => Key::KeyW,
        KeyX => Key::KeyX,
        KeyY => Key::KeyY,
        KeyZ => Key::KeyZ,
        Digit0 | Numpad0 => Key::Key0,
        Digit1 | Numpad1 => Key::Key1,
        Digit2 | Numpad2 => Key::Key2,
        Digit3 | Numpad3 => Key::Key3,
        Digit4 | Numpad4 => Key::Key4,
        Digit5 | Numpad5 => Key::Key5,
        Digit6 | Numpad6 => Key::Key6,
        Digit7 | Numpad7 => Key::Key7,
        Digit8 | Numpad8 => Key::Key8,
        Digit9 | Numpad9 => Key::Key9,
        Escape => Key::KeyEscape,
        Backspace => Key::KeyBackspace,
        Tab => Key::KeyTab,
        Space => Key::KeySpace,
        Enter => Key::KeyEnter,
        Backquote => Key::KeyGrave,
        Minus | NumpadSubtract => Key::KeyMinus,
        Equal => Key::KeyEquals,
        BracketLeft => Key::KeyLeftBracket,
        BracketRight => Key::KeyRightBracket,
        Semicolon => Key::KeySemicolon,
        Quote => Key::KeySingleQuote,
        Comma => Key::KeyComma,
        Period => Key::KeyPeriod,
        Slash => Key::KeySlash,
        Backslash => Key::KeyBackslash,
        Home => Key::KeyHome,
        End => Key::KeyEnd,
        Insert => Key::KeyInsert,
        Delete => Key::KeyDelete,
        PageDown => Key::KeyPageDown,
        PageUp => Key::KeyPageUp,
        ArrowUp => Key::KeyUp,
        ArrowDown => Key::KeyDown,
        ArrowLeft => Key::KeyLeft,
        ArrowRight => Key::KeyRight,
        F1 => Key::KeyF1,
        F2 => Key::KeyF2,
        F3 => Key::KeyF3,
        F4 => Key::KeyF4,
        F5 => Key::KeyF5,
        F6 => Key::KeyF6,
        F7 => Key::KeyF7,
        F8 => Key::KeyF8,
        F9 => Key::KeyF9,
        F10 => Key::KeyF10,
        F11 => Key::KeyF11,
        F12 => Key::KeyF12,
        _ => return None,
    };

    let state = if event.state == ElementState::Pressed {
        InputActionState::Started
    } else {
        InputActionState::Stopped
    };

    Some(KeyboardEvent { key, state })
}