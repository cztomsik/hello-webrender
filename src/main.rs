extern crate webrender;
// Renderer::new takes gleam::gl::Gl
extern crate gleam;
extern crate glutin;

//use webrender::api::{BuiltDisplayList, DeviceUintSize, DocumentId, LayoutSize, PipelineId, RenderApi, RenderNotifier};
use webrender::api::{RenderNotifier, DocumentId};

use glutin::GlContext;

use webrender::api::*;

struct Notifier (glutin::EventsLoopProxy);

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<RenderNotifier> {
        return Box::new(Notifier(self.0.clone()));
    }

    fn wake_up(&self) {
        self.0.wakeup();
    }

    fn new_frame_ready(&self, _docId: DocumentId, _scrolled: bool, _composite_needed: bool, _render_time_ns: Option<u64>) {
        // TODO
        // self.0.wake_up();
    }
}

fn main() {
    let gl_version = glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2));
    let context = glutin::ContextBuilder::new().with_gl(gl_version).with_vsync(true);

    let mut event_loop = glutin::EventsLoop::new();
    let window = glutin::GlWindow::new(glutin::WindowBuilder::new().with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0)), context, &event_loop).unwrap();

    unsafe {
        window.make_current().ok();
    }

    println!("window ok");

    let gl = unsafe { gleam::gl::GlFns::load_with(|s| window.context().get_proc_address(s) as *const std::os::raw::c_void) };
    println!("gl ok {}", gl.get_string(gleam::gl::VERSION));

    window.show();

    println!("window shown");

    let device_pixel_ratio = window.get_hidpi_factor() as f32;
    println!("Device pixel ratio: {}", device_pixel_ratio);

    let opts = webrender::RendererOptions {
        //precache_flags: E::PRECACHE_SHADER_FLAGS,
        device_pixel_ratio,
        clear_color: Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
        //scatter_gpu_cache_updates: false,
        debug_flags: webrender::DebugFlags::ECHO_DRIVER_MESSAGES,
        ..webrender::RendererOptions::default()
    };

    let (mut _renderer, _sender) = webrender::Renderer::new(
        gl,
        Box::new(Notifier(event_loop.create_proxy())),
        opts,
        Option::None
    ).unwrap();

    println!("renderer ok");

    event_loop.run_forever(|_event| {
        glutin::ControlFlow::Continue
    });

    println!("running loop");
}
