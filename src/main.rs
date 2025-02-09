mod view;
use view::View;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub use tokio;
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
    let (mut view, event_loop) = View::new(800, 600);
    setup(&mut view).await; // Clone `view` for use in async context

    let window = view.window.clone(); // Clone the `Arc<Window>` for use in the event loop

    event_loop.run(move |event, _| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    println!("Closing window...");
                    std::process::exit(0);
                }
                _ => {}
            },
            winit::event::Event::AboutToWait => {
                window.request_redraw(); // ✅ Use `window` to request redraw
            }
            _ => {}
        }
    });
}

/// **WebAssembly Main Function**
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    spawn_local(async move {
        let view = Rc::new(RefCell::new(View::new(800, 600))); // ✅ Wrap in `Rc<RefCell<View>>`
        setup(&mut view.borrow_mut()).await;

        let view_clone = view.clone(); // ✅ Clone for closure
        let performance = window()
            .unwrap()
            .performance()
            .expect("❌ Performance API not available");
        let last_time = Rc::new(Cell::new(performance.now())); // ✅ Track last timestamp

        let closure = Rc::new(RefCell::new(None));
        let closure_clone = closure.clone();

        *closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let now = performance.now();
            let elapsed_time = (now - last_time.get()) as f32 / 1000.0; // ✅ Convert to seconds
            last_time.set(now);

            let view = view_clone.borrow_mut(); // ✅ Access `view` mutably
            view.render_frame(elapsed_time);

            request_animation_frame(closure_clone.borrow().as_ref().unwrap());
        }) as Box<dyn Fn()>));

        request_animation_frame(closure.borrow().as_ref().unwrap());
        closure.borrow_mut().take().unwrap().forget(); // ✅ Keep closure alive
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
