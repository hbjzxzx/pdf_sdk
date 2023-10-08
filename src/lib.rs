use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use pathfinder_geometry::vector::{vec2i, vec2f};
use pathfinder_geometry::rect::RectF;
use pathfinder_gl::{GLDevice, GLVersion};

use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererMode, RendererOptions};
use pathfinder_renderer::gpu::renderer::Renderer;
use pathfinder_renderer::options::BuildOptions;
use pathfinder_renderer::scene::Scene;

use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
use pathfinder_renderer::concurrent::rayon::RayonExecutor;

use pathfinder_resources::embedded::EmbeddedResourceLoader;

use pathfinder_canvas::{Canvas, CanvasFontContext, Path2D, CanvasRenderingContext2D, Vector2F, Transform2F};

use pathfinder_color::ColorF;


#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use pdf::{file::{FileOptions, SyncCache, NoLog}, object::{PlainRef, PageRc}, any::AnySync, PdfError};
use pdf_render::render_page;

use std::{path::PathBuf, sync::Arc};

pub mod render;

// fn render_on_canvas(frame_size: &Vector2F) -> CanvasRenderingContext2D{
//      // start to render
//     let font_context = CanvasFontContext::from_system_source();
//     // Make a canvas. We're going to draw a house.
//     // let mut canvas = Canvas::new(frame_size.clone()).get_context_2d(font_context.clone());
//
//     let scene: Scene = Scene::new();
//     let mut rcanvas = Canvas::from_scene(scene);
//     rcanvas.set_size(frame_size.to_i32());
//     let mut canvas = rcanvas.get_context_2d(font_context);
//     // let mut canvas = Canvas::new(frame_size.clone()).get_context_2d(font_context.clone());
//
//     // Set line width.
//     canvas.set_line_width(10.0);
//
//     // Draw walls.
//     canvas.stroke_rect(RectF::new(vec2f(75.0, 140.0), vec2f(150.0, 110.0)));
//
//     // Draw door.
//     canvas.fill_rect(RectF::new(vec2f(130.0, 190.0), vec2f(40.0, 60.0)));
//
//     // Draw roof.
//     let mut path = Path2D::new();
//     path.move_to(vec2f(50.0, 140.0));
//     path.line_to(vec2f(150.0, 60.0));
//     path.line_to(vec2f(250.0, 140.0));
//     path.close_path();
//     canvas.stroke_path(path);
//
//     canvas
// }
//


pub fn render_to_window(window: &Window) -> impl FnMut() -> () {
    #[cfg (not(target_arch = "wasm32"))] 
    use crate::render::pc::render_to_window_imp;

    #[cfg (target_arch = "wasm32")] 
    use crate::render::web::render_to_window_imp;
    
    return render_to_window_imp(window)
}


fn get_window_builder() -> winit::window::WindowBuilder {
    let builder = WindowBuilder::new();

    #[cfg(target_arch = "wasm32")] 
    let builder = {
        use winit::platform::web::WindowBuilderExtWebSys;
        builder.with_append()
    };
    builder
}
pub fn run() {
    let event_loop = EventLoop::new();
    
    let wnd_builder = get_window_builder(); 
    let window = wnd_builder 
        .with_title("The Mini Pdf Viewer")
        .with_inner_size(LogicalSize::new(800, 800))
        .build(&event_loop)
        .unwrap();
    // #[cfg(wasm_platform)]
    // {
    // use winit::platform::web;
    // WindowBuilder::new().with_canvas();
    // }
    let mut need_render = true;
    let mut cf = None;
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();
        if need_render {
            cf = Some(render_to_window(&window));
            need_render = false;
        }
        match event {
            Event::WindowEvent {
                window_id: _wid,
                event: wevent,
            } => {
                if let WindowEvent::CloseRequested = wevent {
                    cf.as_mut().take().unwrap();
                    println!("normal closing!!!!");
                    control_flow.set_exit();
                }
            }
            Event::MainEventsCleared => {
                // window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                println!("receive redraw event");
            }
            _e => {
                // println!("receive event: {:?}", e);
            },
        }

    });

}
