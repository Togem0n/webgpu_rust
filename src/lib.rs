#![allow(dead_code)]
use std:: {iter};
use wgpu::util::DeviceExt; use cgmath::*;
use winit::{
event::*,
event_loop::{ControlFlow, EventLoop}, window::{Window, WindowBuilder},
};
use bytemuck:: {cast_slice}; 

#[path="./common/common.rs"]
pub mod common;
#[path="./common/vertex_data.rs"]
pub mod vertex_data;
// const IS_PERSPECTIVE:bool = false;

const ROTATION_SPEED: f32 = 1.0;
const NX: usize = 4;
const NY: usize = 4;
const NXY: usize = 16;

struct State {
    init: common::InitWgpu, 
    pipeline: wgpu::RenderPipeline, 
    vertex_buffer: wgpu::Buffer, 
    index_buffer: wgpu::Buffer,
    indices_len: u32,
    uniform_buffer: wgpu::Buffer, 
    uniform_bind_group:wgpu::BindGroup, 
    model_mat: Matrix4<f32>,
    view_mat: Matrix4<f32>, 
    project_mat: Matrix4<f32>,
}

impl State {
    async fn new (window: &Window) -> Self{
        let init = common::InitWgpu::init_wgpu(window).await;
        let shader = init.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shader/cube.wgsl").into()),   
        });

        // init mvp matrix
        let camera_position = (0.0, 0.0, 4.5).into(); 
        let look_direction = (0.0,0.0,0.0).into(); 
        let up_direction = cgmath::Vector3::unit_y();

        let model_mat = common::create_transforms([0.0,0.0,0.0], [0.0,0.0,0.0], [1.0,1.0,1.0]); 
        let (view_mat, project_mat, view_project_mat) =
            common::create_view_projection(
                                            camera_position, 
                                            look_direction, 
                                            up_direction,
                                    init.config.width as f32 / init.config.height as f32, 
                            true); 
        
        let matrix_size = 16 * 4;
        let uniform_buffer_size = NXY as u64 * matrix_size;

        // pass mvp to uniform buffer
        let uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor{ 
            label: Some("Uniform Buffer"),
            size: uniform_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
        });

        // let mvp_mat = view_project_mat * model_mat;
        // let mvp_ref:&[f32; 16] = mvp_mat.as_ref();
        
        // let uniform_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Uniform Buffer"),
        //     contents: bytemuck::cast_slice(mvp_ref),
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        // });

        let uniform_bind_group_layout = init.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{ 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: None,
                },
                count: None, 
            }],
            label: Some("Uniform Bind Group Layout"), 
        });

        let uniform_bind_group = init.device.create_bind_group(&wgpu::BindGroupDescriptor{
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(), 
            }],
            label: Some("Uniform Bind Group"), 
        });

        let pipeline_layout = init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = init.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_data::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: init.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState{
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus, 
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual, 
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(), 
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let (vertices, indices) = vertex_data::create_cube_vertices_with_indices();
        let vertex_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
            label: Some("Vertex Buffer"),
            contents: cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Indices Buffer"),
            contents: cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX, 
        });

        let indices_len = indices.len() as u32;

        Self{
            init,
            pipeline, 
            vertex_buffer, 
            index_buffer,
            indices_len,
            uniform_buffer, 
            uniform_bind_group, 
            model_mat, 
            view_mat, 
            project_mat,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.init.size = new_size;
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init.surface.configure(&self.init.device, &self.init.config);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self, dt: std::time::Duration) {
        let dt = ROTATION_SPEED * dt.as_secs_f32(); 
        
        let mut mvp_mat:Vec<[f32;16]> = Vec::with_capacity(NXY as usize);

        for x in 0..NX {
            for y in 0..NY {
                let xx = 1.5 * (x as f32 - NX as f32/2.0 + 0.5); 
                let yy = 1.5 * (y as f32 - NY as f32/2.0 + 0.5); 
                let tx = dt * (x as f32 + 0.5);
                let ty = dt * (y as f32 + 0.5);
                let model_mat = common::create_transforms(
                    [xx,yy,0.0], [tx.sin(), ty.cos(), 0.0], [0.3, 0.3, 0.3]);
                let mvp = self.project_mat * self.view_mat * model_mat; let mvp1:&[f32;16] = mvp.as_ref();
                mvp_mat.push(*mvp1);
            }
        }
        let mvp_ref:&[[f32; 16]; NXY] = mvp_mat[..].try_into().unwrap();
        self.init.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(mvp_ref));
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        //let output = self.init.surface.get_current_frame()?.output;
        let output = self.init.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = self.init.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.init.config.width,
                height: self.init.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format:wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .init.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..self.indices_len, 0, 0..NXY as u32);
        }

        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

}

pub async fn run(){
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title(&*format!("{}", "webgpu_rust"));
    let mut state = pollster::block_on(State::new(&window));
    let render_start_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - render_start_time;
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.init.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}