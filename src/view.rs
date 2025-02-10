use glow::HasContext;
use image::DynamicImage;
use std::collections::HashMap;
use std::sync::Arc;

// 🖥️ **Native (Desktop) Imports**
#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub use std::ffi::CString;
    pub use std::num::NonZeroU32;

    pub use glutin::{
        config::ConfigTemplateBuilder,
        context::{
            ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentGlContext,
            PossiblyCurrentContext, Version,
        },
        display::{GetGlDisplay, GlDisplay},
        surface::{SurfaceAttributesBuilder, WindowSurface},
    };
    pub use glutin_winit::DisplayBuilder;

    pub use winit::dpi::PhysicalSize;
    pub use winit::event_loop::EventLoop;
    pub use winit::window::{Window, WindowAttributes};
}

// 🌍 **Web (WASM) Imports**
#[cfg(target_arch = "wasm32")]
mod web {
    pub use wasm_bindgen::JsCast;
    pub use wasm_bindgen_futures::JsFuture;
    pub use web_sys::{console, window, HtmlCanvasElement, Response, WebGl2RenderingContext};
}

// ✅ Import conditionally based on platform
#[cfg(not(target_arch = "wasm32"))]
use native::*;
#[cfg(target_arch = "wasm32")]
use web::*;

pub struct View {
    gl: Arc<glow::Context>,
    textures: HashMap<String, glow::Texture>,
    width: u32,
    height: u32,

    #[cfg(not(target_arch = "wasm32"))]
    pub window: Arc<Window>,
    #[cfg(not(target_arch = "wasm32"))]
    pub gl_context: glutin::context::PossiblyCurrentContext,
    #[cfg(not(target_arch = "wasm32"))]
    pub surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl View {
    /// **Creates a new View and initializes OpenGL/WebGL**
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(width: u32, height: u32) -> (Self, EventLoop<()>) {
        use winit::raw_window_handle::HasRawWindowHandle;

        let event_loop = EventLoop::new().expect("Failed to create EventLoop");

        let window_attributes = WindowAttributes::default()
            .with_title("Rust OpenGL Window")
            .with_inner_size(PhysicalSize::new(width, height));

        // ✅ Use DisplayBuilder to create the window + OpenGL display
        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes))
            .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| {
                configs.next().unwrap()
            })
            .unwrap();

        let window = Arc::new(window.unwrap()); // Unwrap because `Some` window exists

        // ✅ Define OpenGL context attributes
        let raw_context = ContextAttributesBuilder::new()
            .with_profile(GlProfile::Core)
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3)))) // OpenGL 3.3 Core Profile
            .build(Some(window.raw_window_handle().unwrap()));

        // ✅ Create OpenGL context
        let not_current_gl_context = unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &raw_context)
                .unwrap()
        };

        // ✅ Create a surface
        let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window.raw_window_handle().unwrap(),
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );
        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &surface_attributes)
                .unwrap()
        };

        let gl_context = not_current_gl_context.make_current(&surface).unwrap();

        // ✅ Load OpenGL function pointers
        let gl = Arc::new(unsafe {
            glow::Context::from_loader_function(|s| {
                let cstr = CString::new(s).unwrap();
                gl_config.display().get_proc_address(&cstr)
            })
        });

        unsafe {
            gl.viewport(0, 0, width as i32, height as i32); // ✅ Ensure viewport matches window
        }

        (
            Self {
                gl,
                gl_context,
                height,
                surface,
                textures: HashMap::new(),
                width,
                window,
            },
            event_loop,
        )
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(width: u32, height: u32) -> Self {
        console::log_1(&"✅ View::new() - Starting initialization".into());

        let window = match window() {
            Some(win) => {
                console::log_1(&"✅ window() returned successfully".into());
                win
            }
            None => {
                console::log_1(
                    &"❌ ERROR: `window()` returned None. Are you running in a browser?".into(),
                );
                panic!("❌ ERROR: `window()` returned None. Are you running in a browser?");
            }
        };

        let document = match window.document() {
            Some(doc) => {
                console::log_1(&"✅ document() returned successfully".into());
                doc
            }
            None => {
                console::log_1(&"❌ ERROR: `document()` returned None. Is JavaScript blocking access to the DOM?".into());
                panic!("❌ ERROR: `document()` returned None. Is JavaScript blocking access to the DOM?");
            }
        };

        let canvas = match document.get_element_by_id("canvas") {
            Some(c) => {
                console::log_1(&"✅ Canvas element found in the DOM".into());
                c.dyn_into::<HtmlCanvasElement>().unwrap()
            }
            None => {
                console::log_1(
                    &"❌ ERROR: Canvas element with id 'canvas' not found in the DOM".into(),
                );
                panic!("❌ ERROR: Canvas element with id 'canvas' not found in the DOM");
            }
        };

        canvas.set_width(width);
        canvas.set_height(height);

        let webgl_context = match canvas.get_context("webgl2") {
            Ok(Some(ctx)) => {
                console::log_1(&"✅ WebGL2 context created successfully".into());
                ctx.dyn_into::<WebGl2RenderingContext>().unwrap()
            }
            _ => {
                console::log_1(&"❌ ERROR: WebGL2 context could not be created. Your browser may not support WebGL2.".into());
                panic!("❌ ERROR: WebGL2 context could not be created. Your browser may not support WebGL2.");
            }
        };

        console::log_1(&"✅ Glow WebGL context initialization successful".into());

        let gl = Arc::new(glow::Context::from_webgl2_context(webgl_context));

        Self {
            gl,
            textures: HashMap::new(),
            width,
            height,
        }
    }

    /// **Handles window resizing**
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.window
                .request_inner_size(winit::dpi::PhysicalSize::new(new_width, new_height));
        }

        #[cfg(target_arch = "wasm32")]
        {
            let canvas = window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();

            canvas.set_width(new_width);
            canvas.set_height(new_height);
        }
    }

    /// **Loads a texture asynchronously and caches it by path (Native)**
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn load_texture(&mut self, path: &str) {
        if self.textures.contains_key(path) {
            return; // Texture is already loaded
        }

        let response = reqwest::get(path).await.expect("Failed to fetch image");
        let bytes = response.bytes().await.expect("Failed to read image bytes");

        let img = image::load_from_memory(&bytes).expect("Failed to decode image");
        let (width, height, data) = Self::decode_image(img);

        let texture = self.upload_texture(width, height, &data);
        self.textures.insert(path.to_string(), texture);
    }

    /// **Loads a texture asynchronously and caches it by URL (WebAssembly)**
    #[cfg(target_arch = "wasm32")]
    pub async fn load_texture(&mut self, url: &str) {
        if self.textures.contains_key(url) {
            return; // Texture is already loaded
        }

        let response: Response = JsFuture::from(window().unwrap().fetch_with_str(url))
            .await
            .expect("Failed to fetch image")
            .dyn_into()
            .unwrap();

        let buffer = JsFuture::from(response.array_buffer().unwrap())
            .await
            .expect("Failed to get array buffer");

        let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
        let img = image::load_from_memory(&bytes).expect("Failed to decode image");

        let (width, height, data) = Self::decode_image(img);

        let texture = self.upload_texture(width, height, &data);
        self.textures.insert(url.to_string(), texture);
    }

    fn decode_image(img: DynamicImage) -> (u32, u32, Vec<u8>) {
        let img = img.to_rgba8();
        let (width, height) = img.dimensions();
        (width, height, img.into_raw())
    }

    fn upload_texture(&self, width: u32, height: u32, data: &[u8]) -> glow::Texture {
        unsafe {
            let texture = self.gl.create_texture().expect("Failed to create texture");
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );

            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(data)),
            );

            texture
        }
    }

    pub fn bind_texture(&self, path: &str) {
        unsafe {
            if let Some(texture) = self.textures.get(path) {
                self.gl.bind_texture(glow::TEXTURE_2D, Some(*texture));
            }
        }
    }

    pub fn render_frame(&self, time_ms: f64) {
        let time_sec = time_ms / 1000.0; // ✅ Convert milliseconds to seconds

        let r = (time_sec.sin() * 0.5 + 0.5) as f32; // ✅ Cycles every 2π seconds
        let g = (time_sec.cos() * 0.5 + 0.5) as f32; // ✅ Cycles every 2π seconds
        let b = 0.5; // Static blue

        unsafe {
            self.gl.clear_color(r, g, b, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }
}
