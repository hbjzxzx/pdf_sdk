use pathfinder_canvas::{Canvas,
                        CanvasFontContext,
                        Path2D,
                        CanvasRenderingContext2D, Vector2F, Transform2F};
use pathfinder_renderer::scene::Scene;

use pdf::{file::{FileOptions, SyncCache, NoLog}, object::{PlainRef, PageRc}, any::AnySync, PdfError};
use pdf_render::render_page;


#[cfg(target_arch = "wasm32")]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}


pub fn render_pdf_on_canvas(frame_size: &Vector2F,
                        resolver: impl pdf::object::Resolve,
                        page: PageRc,
) -> CanvasRenderingContext2D {
    // start to render
    let font_context = CanvasFontContext::from_system_source();
    // Make a canvas. We're going to draw a house.
    // let mut canvas = Canvas::new(frame_size.clone()).get_context_2d(font_context.clone());

    let mut secen = Scene::new();
    let cache = pdf_render::Cache::new();
    let mut backend = pdf_render::SceneBackend::new(cache, &mut secen);

    render_page(&mut backend,
                &resolver,
                &page,
                Transform2F::from_scale(150.0 / 25.4)).unwrap();
    let mut canvas = Canvas::from_scene(secen);
    canvas.set_size(frame_size.to_i32());
    canvas.get_context_2d(font_context)
}