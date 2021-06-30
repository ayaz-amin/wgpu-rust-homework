use futures::executor::block_on;
use std::borrow::Cow;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window}
};

struct State
{
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain_desc: wgpu::SwapChainDescriptor,
    swapchain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    render_pipeline_2: wgpu::RenderPipeline,
    render_pipeline_switch: bool,
    color: wgpu::Color
}

impl State
{
    async fn new(window: &Window) -> Self
    {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::DX12);
        let surface = unsafe{instance.create_surface(window)};
        
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions
            {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor
            {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default()
            },
            None
        ).await.unwrap();

        let swapchain_desc = wgpu::SwapChainDescriptor
        {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swapchain = device.create_swap_chain(&surface, &swapchain_desc);

        let _shader = device.create_shader_module(
            &wgpu::ShaderModuleDescriptor
            {
                label: Some("Shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl")))
            }
        );

        let _shader_2 = device.create_shader_module(
            &wgpu::ShaderModuleDescriptor
            {
                label: Some("Shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader2.wgsl")))
            }
        );

        let _render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor
            {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[]
            }
        );
        
        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor
            {
                label: Some("Render pipeline"),
                layout: Some(&_render_pipeline_layout),
                vertex: wgpu::VertexState
                {
                    module: &_shader,
                    entry_point: "main",
                    buffers: &[]
                },
                fragment: Some(wgpu::FragmentState
                {
                    module: &_shader,
                    entry_point: "main",
                    targets: &[
                        wgpu::ColorTargetState
                        {
                            format: swapchain_desc.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrite::ALL
                        }
                    ]
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default()
            }
        );

        let render_pipeline_2 = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor
            {
                label: Some("Render pipeline"),
                layout: Some(&_render_pipeline_layout),
                vertex: wgpu::VertexState
                {
                    module: &_shader_2,
                    entry_point: "main",
                    buffers: &[]
                },
                fragment: Some(wgpu::FragmentState
                {
                    module: &_shader_2,
                    entry_point: "main",
                    targets: &[
                        wgpu::ColorTargetState
                        {
                            format: swapchain_desc.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrite::ALL
                        }
                    ]
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default()
            }
        );

        let color = wgpu::Color{r: 0.1, g: 0.2, b: 0.3, a: 1.0};
        let render_pipeline_switch = false;

        return Self
        {
            surface,
            device,
            queue,
            swapchain_desc,
            swapchain,
            size,
            render_pipeline,
            render_pipeline_2,
            render_pipeline_switch,
            color
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>)
    {
        self.size = new_size;
        self.swapchain_desc.width = new_size.width;
        self.swapchain_desc.height = new_size.height;
        self.swapchain = self.device.create_swap_chain(&self.surface, &self.swapchain_desc);
    }

    fn input(&mut self, event: &WindowEvent) -> bool
    {
        match event
        {
            WindowEvent::CursorMoved{position, ..} =>
            {
                let _pos_x = position.x as f64 / self.size.width as f64;
                let _pos_y = position.y as f64 / self.size.height as f64;
                self.color = wgpu::Color{r: _pos_x, g: _pos_y, b: _pos_x + _pos_y, a: 1.0};
                true
            },
            _ => false
        }
    }

    fn update(&self)
    {

    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError>
    {
        let frame = self.swapchain.get_current_frame()?.output;
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor
            {
                label: Some("render encoder")
            }
        );
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor
                {
                    label: Some("render pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachment
                    {
                        view: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations
                        {
                            load: wgpu::LoadOp::Clear(self.color),
                            store: true
                        }
                    }],
                    depth_stencil_attachment: None
                }
            );
            if self.render_pipeline_switch == true
            {
                render_pass.set_pipeline(&self.render_pipeline_2);
            } else {
                render_pass.set_pipeline(&self.render_pipeline);
            }
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        return Ok(());
    }
}

fn main()
{
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = block_on(State::new(&window));

    event_loop.run(
        move |event, _, control_flow|
        {
            *control_flow = ControlFlow::Wait;
            match event
            {
                Event::WindowEvent
                {
                    ref event,
                    window_id
                } if window_id == window.id() => if !state.input(event)
                {
                    match event
                    {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput
                        {
                            input,
                            ..
                        } => {
                            match input
                            {
                                KeyboardInput
                                {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                } => *control_flow = ControlFlow::Exit,
                                KeyboardInput
                                {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Space),
                                    ..
                                } => state.render_pipeline_switch = !state.render_pipeline_switch,
                                _ => ()
                            }
                        },
                        WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                        WindowEvent::ScaleFactorChanged{new_inner_size, ..} => state.resize(**new_inner_size),
                        _ => ()
                    }
                },
                Event::RedrawRequested(_) =>
                {
                    state.update();
                    match state.render()
                    {
                        Ok(_) => {},
                        Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                        Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e)
                    }
                }
                Event::MainEventsCleared =>
                {
                    window.request_redraw();
                }
                _ => ()
            }
        }
    )
}