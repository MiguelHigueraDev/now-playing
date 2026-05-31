use std::ffi::CStr;
use std::num::NonZeroU32;
use std::sync::Arc;

use egui_winit::winit;
use winit::raw_window_handle::HasWindowHandle;

pub struct GlutinWindowContext {
    pub window: winit::window::Window,
    gl_context: glutin::context::PossiblyCurrentContext,
    gl_display: glutin::display::Display,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl GlutinWindowContext {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        title: &str,
        width: f64,
        height: f64,
    ) -> Self {
        use glutin::context::NotCurrentGlContext;
        use glutin::display::{GetGlDisplay, GlDisplay};
        use glutin::prelude::GlSurface;

        let winit_window_builder = winit::window::WindowAttributes::default()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize { width, height })
            .with_resizable(false);

        let config_template_builder = glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(None)
            .with_depth_size(0)
            .with_stencil_size(0)
            .with_transparency(false);

        let (mut window, gl_config) = glutin_winit::DisplayBuilder::new()
            .with_preference(glutin_winit::ApiPreference::FallbackEgl)
            .with_window_attributes(Some(winit_window_builder.clone()))
            .build(
                event_loop,
                config_template_builder,
                |mut config_iterator| {
                    config_iterator
                        .next()
                        .expect("failed to find a matching GL configuration")
                },
            )
            .expect("failed to create GL config");

        let gl_display = gl_config.display();

        let raw_window_handle = window.as_ref().map(|w| {
            w.window_handle()
                .expect("failed to get window handle")
                .as_raw()
        });

        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(raw_window_handle);
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(raw_window_handle);

        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_config
                        .display()
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create GL context")
                })
        };

        let window = window.take().unwrap_or_else(|| {
            glutin_winit::finalize_window(event_loop, winit_window_builder, &gl_config)
                .expect("failed to finalize glutin window")
        });

        let (width, height): (u32, u32) = window.inner_size().into();
        let width = NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN);
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window
                        .window_handle()
                        .expect("failed to get window handle")
                        .as_raw(),
                    width,
                    height,
                );

        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .unwrap()
        };

        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();
        gl_surface
            .set_swap_interval(
                &gl_context,
                glutin::surface::SwapInterval::Wait(NonZeroU32::MIN),
            )
            .unwrap();

        Self {
            window,
            gl_context,
            gl_display,
            gl_surface,
        }
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface;

        self.gl_surface.resize(
            &self.gl_context,
            physical_size.width.try_into().unwrap(),
            physical_size.height.try_into().unwrap(),
        );
    }

    pub fn swap_buffers(&self) -> glutin::error::Result<()> {
        use glutin::surface::GlSurface;

        self.gl_surface.swap_buffers(&self.gl_context)
    }

    fn get_proc_address(&self, addr: &CStr) -> *const std::ffi::c_void {
        use glutin::display::GlDisplay;

        self.gl_display.get_proc_address(addr)
    }
}

pub fn create_gl_context(gl_window: &GlutinWindowContext) -> Arc<glow::Context> {
    Arc::new(unsafe {
        glow::Context::from_loader_function(|s| {
            let name = std::ffi::CString::new(s).expect("invalid GL proc name");
            gl_window.get_proc_address(&name)
        })
    })
}
