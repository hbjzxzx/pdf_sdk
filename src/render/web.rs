use winit::window::Window;

pub fn render_to_window_imp(windows: &Window){
    crate::render::utils::set_panic_hook();
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
