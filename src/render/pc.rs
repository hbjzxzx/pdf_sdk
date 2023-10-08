use std::{path::PathBuf, sync::Arc};

use winit::window::Window;
use surfman::{
    Connection, ContextAttributeFlags, ContextAttributes,
    GLVersion as SurfmanGLVersion
};
use surfman::{SurfaceAccess, SurfaceType};

use pathfinder_geometry::vector::{vec2f, vec2i};
use pathfinder_gl::{GLDevice, GLVersion};
use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererMode, RendererOptions};
use pathfinder_renderer::gpu::renderer::Renderer;
use pathfinder_renderer::options::BuildOptions;
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use pathfinder_color::ColorF;

use pdf::{file::{FileOptions, SyncCache, NoLog}, object::{PlainRef, PageRc}, any::AnySync, PdfError};
use pdf_render::render_page;

type PdfFile = pdf::file::File<Vec<u8>,
    Arc<SyncCache<PlainRef, Result<AnySync, Arc<PdfError>>>>,
    Arc<SyncCache<PlainRef, Result<Arc<[u8]>, Arc<PdfError>>>>,
    NoLog>;

use crate::render::utils::render_pdf_on_canvas;

// type PdfResolver = pdf
fn read_pdf_file(path: PathBuf) -> PdfFile {
    println!("reading pdf from file: {}", path.to_str().unwrap());
    let file = FileOptions::cached().open(path).unwrap();
    return file;
}


pub fn render_to_window_imp(window: &Window) -> impl FnMut() -> () {

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
    use std::env::current_dir;
    
    let pdf_file = read_pdf_file(current_dir().unwrap().join("compressed.tracemonkey-pldi-09.pdf"));
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
