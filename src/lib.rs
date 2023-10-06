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


fn render_on_canvas(frame_size: &Vector2F) -> CanvasRenderingContext2D{
     // start to render
    let font_context = CanvasFontContext::from_system_source();
    // Make a canvas. We're going to draw a house.
    // let mut canvas = Canvas::new(frame_size.clone()).get_context_2d(font_context.clone());
    
    let scene: Scene = Scene::new();
    let mut rcanvas = Canvas::from_scene(scene);
    rcanvas.set_size(frame_size.to_i32());
    let mut canvas = rcanvas.get_context_2d(font_context);
    // let mut canvas = Canvas::new(frame_size.clone()).get_context_2d(font_context.clone());

    // Set line width.
    canvas.set_line_width(10.0);

    // Draw walls.
    canvas.stroke_rect(RectF::new(vec2f(75.0, 140.0), vec2f(150.0, 110.0)));

    // Draw door.
    canvas.fill_rect(RectF::new(vec2f(130.0, 190.0), vec2f(40.0, 60.0)));

    // Draw roof.
    let mut path = Path2D::new();
    path.move_to(vec2f(50.0, 140.0));
    path.line_to(vec2f(150.0, 60.0));
    path.line_to(vec2f(250.0, 140.0));
    path.close_path();
    canvas.stroke_path(path);
    
    canvas
}

type PdfFile = pdf::file::File<Vec<u8>, 
                                Arc<SyncCache<PlainRef, Result<AnySync, Arc<PdfError>>>>, 
                                Arc<SyncCache<PlainRef, Result<Arc<[u8]>, Arc<PdfError>>>>, 
                                NoLog>;
                                
// type PdfResolver = pdf
fn read_pdf_file(path: PathBuf) -> PdfFile {
    println!("reading pdf from file: {}", path.to_str().unwrap());
    let file = FileOptions::cached().open(path).unwrap();
    return file;
}

fn render_pdf_on_canvas(frame_size: &Vector2F, 
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

pub fn render_to_window(window: &Window) -> impl FnMut() -> () {
    #[cfg (not(target_arch = "wasm32"))] 
    return render_to_window_pc(window);  

    #[cfg (target_arch = "wasm32")] 
    return render_to_windows_web(window);
}

#[cfg (not(target_arch = "wasm32"))]
pub fn render_to_window_pc(window: &Window) -> impl FnMut() -> () {
    use surfman::{
        Connection, ContextAttributeFlags, ContextAttributes, GLVersion as SurfmanGLVersion,
    };
    use surfman::{SurfaceAccess, SurfaceType};

    // Create a `surfman` device. On a multi-GPU system, we'll request the low-power integrated
    // GPU.
    let connection = Connection::from_winit_window(&window).unwrap();
    let native_widget = connection
        .create_native_widget_from_winit_window(&window)
        .unwrap();
    let adapter = connection.create_low_power_adapter().unwrap();
    let mut device = connection.create_device(&adapter).unwrap();

    // Request an OpenGL 3.x context. Pathfinder requires this.
    let context_attributes = ContextAttributes {
        version: SurfmanGLVersion::new(3, 0),
        flags: ContextAttributeFlags::ALPHA,
    };
    let context_descriptor = device
        .create_context_descriptor(&context_attributes)
        .unwrap();

    // Make the OpenGL context via `surfman`, and load OpenGL functions.
    let surface_type = SurfaceType::Widget { native_widget };
    let mut context = device.create_context(&context_descriptor, None).unwrap();
    let surface = device
        .create_surface(&context, SurfaceAccess::GPUOnly, surface_type)
        .unwrap();
    device
        .bind_surface_to_context(&mut context, surface)
        .unwrap();
    device.make_context_current(&context).unwrap();
    gl::load_with(|symbol_name| device.get_proc_address(&context, symbol_name));

    // Get the real size of the window, taking HiDPI into account.
    // let hidpi_factor = window.scale_factor();
    
    let physical_size = window.inner_size();
    let framebuffer_size: pathfinder_canvas::Vector2I = vec2i(physical_size.width as i32, physical_size.height as i32);

    // Create a Pathfinder GL device.
    let default_framebuffer = device
        .context_surface_info(&context)
        .unwrap()
        .unwrap()
        .framebuffer_object;
    let pathfinder_device = GLDevice::new(GLVersion::GL3, default_framebuffer);

    // Create a Pathfinder renderer.
    let mode = RendererMode::default_for_device(&pathfinder_device);
    let options = RendererOptions {
        dest: DestFramebuffer::full_window(framebuffer_size),
        background_color: Some(ColorF::white()),
        ..RendererOptions::default()
    };
    let resource_loader = EmbeddedResourceLoader::new();
    let mut renderer = Renderer::new(pathfinder_device, &resource_loader, mode, options); 

    // let mut canvas = render_on_canvas(&framebuffer_size.to_f32());
    let pdf_file = read_pdf_file(PathBuf::from("/Users/xuzhenxing/Downloads/compressed.tracemonkey-pldi-09.pdf"));
    let pdf_resolver = pdf_file.resolver();
    let mut pdf_pageRc = pdf_file.get_page(2).unwrap(); 
    let mut canvas = render_pdf_on_canvas(&
        framebuffer_size.to_f32(),
        pdf_resolver,
        pdf_pageRc,
        );
     
    // Render the canvas to screen.
    let mut scene = SceneProxy::from_scene(
        canvas.into_canvas().into_scene(),
        renderer.mode().level,
        RayonExecutor,
    );
    scene.build_and_render(&mut renderer, BuildOptions::default());

    // Present the surface.
    let mut surface = device
        .unbind_surface_from_context(&mut context)
        .unwrap()
        .unwrap();
    device.present_surface(&mut context, &mut surface).unwrap();
    device
        .bind_surface_to_context(&mut context, surface)
        .unwrap();

    move || {
        drop(device.destroy_context(&mut context));
    }
}

// #[cfg(target_arch = "wasm32")]
mod utils;

#[cfg(target_arch = "wasm32")]
pub fn render_to_windows_web(windows: &Window){
    utils::set_panic_hook();
    use winit::platform::web::WindowExtWebSys;
    let canvas = window.canvas().unwrap();
    
    // let mut canvas = render_on_canvas(&framebuffer_size.to_f32());
    let pdf_file = read_pdf_file(PathBuf::from("/Users/xuzhenxing/Downloads/compressed.tracemonkey-pldi-09.pdf"));
    let pdf_resolver = pdf_file.resolver();
    let mut pdf_pageRc = pdf_file.get_page(2).unwrap(); 
    let mut canvas = render_pdf_on_canvas(&
        framebuffer_size.to_f32(),
        pdf_resolver,
        pdf_pageRc,
        ); 

    let mut surface = Surface::from_canvas(canvas.clone()).unwrap();
    surface
    .resize(
        NonZeroU32::new(canvas.width()).unwrap(),
        NonZeroU32::new(canvas.height()).unwrap(),
    )
    .unwrap();
    
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
                    control_flow.set_exit();
                    cf.as_mut().unwrap()();
                    println!("normal closing!!!!");
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
