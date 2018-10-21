extern crate webrender;
// Renderer::new takes gleam::gl::Gl
extern crate gleam;

// window
extern crate glutin;

extern crate euclid;

use webrender::api::{RenderNotifier, DocumentId};

// scope with get_proc_address()
use glutin::GlContext;

use webrender::api::*;

struct Notifier (glutin::EventsLoopProxy);

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<RenderNotifier> {
        return Box::new(Notifier(self.0.clone()));
    }

    fn wake_up(&self) {
        self.0.wakeup().ok();
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

    let (mut renderer, sender) = webrender::Renderer::new(
        gl,
        Box::new(Notifier(event_loop.create_proxy())),
        opts,
        Option::None
    ).unwrap();

    println!("renderer initialized");

    let framebuffer_size = DeviceUintSize::new(800, 600);
    let api = std::rc::Rc::new(sender.create_api());
    let document_id = api.add_document(framebuffer_size, 0);
    let epoch = Epoch(0);
    let pipeline_id = PipelineId(0, 0);
    let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);
    let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
    let mut txn = Transaction::new();

    render(&mut builder);

    println!("render done");

    txn.set_display_list(
        epoch,
        None,
        layout_size,
        builder.finalize(),
        true,
    );
    txn.set_root_pipeline(pipeline_id);
    txn.generate_frame();
    api.send_transaction(document_id, txn);

    println!("frame ok");

    event_loop.run_forever(|_event| {
        let mut txn = Transaction::new();
        let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);

        render(&mut builder);

        txn.set_display_list(
            epoch,
            None,
            layout_size,
            builder.finalize(),
            true,
        );
        txn.generate_frame();
        api.send_transaction(document_id, txn);

        renderer.update();
        renderer.render(framebuffer_size).unwrap();
        let _ = renderer.flush_pipeline_info();
        window.swap_buffers().ok();

        glutin::ControlFlow::Continue
    });

    println!("loop ok");
}

fn render(builder: &mut DisplayListBuilder) {
    let bounds = LayoutRect::new(LayoutPoint::zero(), builder.content_size());
    let info = LayoutPrimitiveInfo::new(bounds);
    builder.push_stacking_context(
        &info,
        None,
        TransformStyle::Flat,
        MixBlendMode::Normal,
        Vec::new(),
        RasterSpace::Screen,
    );

    let mut info = LayoutPrimitiveInfo::new((0, 0).to(50, 50));
    info.tag = Some((0, 2));
    builder.push_rect(&info, ColorF::new(0.0, 0.0, 1.0, 1.0));

    builder.pop_stacking_context();
}

pub trait HandyDandyRectBuilder {
    fn to(&self, x2: i32, y2: i32) -> LayoutRect;
    fn by(&self, w: i32, h: i32) -> LayoutRect;
}
// Allows doing `(x, y).to(x2, y2)` or `(x, y).by(width, height)` with i32
// values to build a f32 LayoutRect
impl HandyDandyRectBuilder for (i32, i32) {
    fn to(&self, x2: i32, y2: i32) -> LayoutRect {
        LayoutRect::new(
            LayoutPoint::new(self.0 as f32, self.1 as f32),
            LayoutSize::new((x2 - self.0) as f32, (y2 - self.1) as f32),
        )
    }

    fn by(&self, w: i32, h: i32) -> LayoutRect {
        LayoutRect::new(
            LayoutPoint::new(self.0 as f32, self.1 as f32),
            LayoutSize::new(w as f32, h as f32),
        )
    }
}
