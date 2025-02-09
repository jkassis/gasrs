mod view;
use view::View;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub use tokio;
    pub use winit::application::ApplicationHandler;
    pub use winit::event_loop::EventLoop;
    pub use winit::event_loop::ActiveEventLoop;
}

#[cfg(target_arch = "wasm32")]
mod web {
    pub use std::cell::Cell;
    pub use std::cell::RefCell;
    pub use std::rc::Rc;
    pub use wasm_bindgen::closure::Closure;
    pub use wasm_bindgen::JsCast;
    pub use wasm_bindgen_futures::spawn_local;
    pub use web_sys::window; // ✅ Import JsCast
}

// ✅ Import conditionally based on platform
#[cfg(not(target_arch = "wasm32"))]
use native::*;
#[cfg(target_arch = "wasm32")]
use web::*;

/// **Common async setup function (Runs in both Native & WebAssembly)**
async fn setup(view: &mut View) {
    // Load a texture asynchronously
    let creditCard = "https://m.media-amazon.com/images/G/01/credit/CBCC/acq-marketing/maple/Q123-1103_US_CBCC_ACQ_Maple_Thumbnail_126x80._CB613265021_.png";
    view.load_texture(creditCard).await;
    view.bind_texture(creditCard);
}

/// **Native Main Function (Uses Tokio)**
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    use std::time::Instant; // ✅ Import Instant for elapsed time tracking

    let (mut view, event_loop) = View::new(800, 600);
    setup(&mut view).await;

    let window = view.window.clone(); // Clone the `Arc<Window>` for use in the event loop
    let start_time = Instant::now(); // ✅ Track start time

    struct MyApp {
        view: View,
        start_time: Instant,
    }

    impl MyApp {
        fn new(_event_loop: &EventLoop<Self>) -> Self {
            let (mut view, _) = View::new(800, 600);
            Self {
                view,
                start_time: Instant::now(),
            }
        }
    }

    impl ApplicationHandler for MyApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
            match event {
                winit::event::WindowEvent::CloseRequested => {
                    println!("Closing window...");
                    event_loop.exit();
                }
                _ => {}
            }
        }

        fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
            let elapsed_time: f64 = self.start_time.elapsed().as_millis() as f64; // ✅ Convert to ms
            self.view.render_frame(elapsed_time);
            self.view.window.request_redraw();
        }
    }
}

/// **WebAssembly Main Function**
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    spawn_local(async move {
        let view = Rc::new(RefCell::new(View::new(800, 600))); // ✅ Use `Rc<RefCell<View>>`
        setup(&mut view.borrow_mut()).await; // ✅ Now mutable borrow works!

        let performance = window()
            .unwrap()
            .performance()
            .expect("❌ Performance API missing");

        let closure_handle = Rc::new(RefCell::new(None)); // ✅ Store closure reference
        let closure_clone = closure_handle.clone();

        // ✅ Define the closure once
        let render_loop = Closure::wrap(Box::new({
            let view = view.clone();
            let closure_clone = closure_clone.clone();

            move || {
                let now = performance.now();
                view.borrow().render_frame(now); // ✅ Call render function

                if let Some(callback) = closure_clone.borrow().as_ref() {
                    request_animation_frame(callback);
                }
            }
        }) as Box<dyn Fn()>);

        // ✅ Store the closure so it stays alive
        *closure_handle.borrow_mut() = Some(render_loop);

        // ✅ Start the animation loop
        request_animation_frame(closure_handle.borrow().as_ref().unwrap());
    });
}

/// ✅ Helper function for `requestAnimationFrame`
#[cfg(target_arch = "wasm32")]
fn request_animation_frame(callback: &Closure<dyn Fn()>) {
    window()
        .unwrap()
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .expect("Failed to request animation frame");
}
