//! Shader page - Julia Set fractal

use icy_ui::wgpu;
use icy_ui::widget::shader::{self, Pipeline, Primitive, Viewport};
use icy_ui::widget::{column, container, row, shader::Shader, slider, space, text};
use icy_ui::{Element, Fill, Rectangle};

use crate::Message;

#[derive(Clone)]
pub struct ShaderState {
    pub c_real: f32,
    pub c_imag: f32,
    pub zoom: f32,
}

impl Default for ShaderState {
    fn default() -> Self {
        Self {
            c_real: -0.7,
            c_imag: 0.27015,
            zoom: 1.0,
        }
    }
}

pub fn update_shader(state: &mut ShaderState, message: &Message) -> bool {
    match message {
        Message::ShaderCRealChanged(value) => {
            state.c_real = *value;
            true
        }
        Message::ShaderCImagChanged(value) => {
            state.c_imag = *value;
            true
        }
        Message::ShaderZoomChanged(value) => {
            state.zoom = *value;
            true
        }
        _ => false,
    }
}

pub fn view_shader(state: &ShaderState) -> Element<'_, Message> {
    let julia = JuliaProgram {
        c_real: state.c_real,
        c_imag: state.c_imag,
        zoom: state.zoom,
    };

    let shader_widget = Shader::new(julia).width(Fill).height(300);

    let c_real_slider = row![
        text("c (real):").width(80),
        slider(-1.5..=1.5, state.c_real, Message::ShaderCRealChanged).step(0.001),
        text(format!("{:.3}", state.c_real)).width(60),
    ]
    .spacing(10);

    let c_imag_slider = row![
        text("c (imag):").width(80),
        slider(-1.5..=1.5, state.c_imag, Message::ShaderCImagChanged).step(0.001),
        text(format!("{:.3}", state.c_imag)).width(60),
    ]
    .spacing(10);

    let zoom_slider = row![
        text("Zoom:").width(80),
        slider(0.1..=4.0, state.zoom, Message::ShaderZoomChanged).step(0.01),
        text(format!("{:.2}x", state.zoom)).width(60),
    ]
    .spacing(10);

    column![
        text("Julia Set Fractal").size(18),
        space().height(10),
        text("A Julia set is a fractal defined by the iteration z = z² + c").size(14),
        space().height(10),
        container(shader_widget)
            .width(Fill)
            .height(300)
            .style(container::bordered_box),
        space().height(20),
        text("Parameters").size(16),
        space().height(10),
        c_real_slider,
        c_imag_slider,
        zoom_slider,
        space().height(10),
        text("Try these presets:").size(14),
        text("• c = -0.7 + 0.27i (dendrite)").size(12),
        text("• c = -0.8 + 0.156i (dragon)").size(12),
        text("• c = -0.4 + 0.6i (rabbit)").size(12),
        text("• c = 0.285 + 0.01i (spiral)").size(12),
    ]
    .spacing(5)
    .into()
}

// =============================================================================
// Julia Set Shader Implementation
// =============================================================================

#[derive(Debug)]
struct JuliaProgram {
    c_real: f32,
    c_imag: f32,
    zoom: f32,
}

impl<Message> shader::Program<Message> for JuliaProgram {
    type State = ();
    type Primitive = JuliaPrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: icy_ui::mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        JuliaPrimitive {
            _bounds: bounds,
            c_real: self.c_real,
            c_imag: self.c_imag,
            zoom: self.zoom,
        }
    }
}

#[derive(Debug)]
struct JuliaPrimitive {
    _bounds: Rectangle,
    c_real: f32,
    c_imag: f32,
    zoom: f32,
}

impl Primitive for JuliaPrimitive {
    type Pipeline = JuliaPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        let uniforms = JuliaUniforms {
            width: bounds.width,
            height: bounds.height,
            c_real: self.c_real,
            c_imag: self.c_imag,
            zoom: self.zoom,
            _padding: [0.0; 3],
        };

        queue.write_buffer(&pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
        true
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct JuliaUniforms {
    width: f32,
    height: f32,
    c_real: f32,
    c_imag: f32,
    zoom: f32,
    _padding: [f32; 3],
}

struct JuliaPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

impl std::fmt::Debug for JuliaPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JuliaPipeline").finish()
    }
}

impl Pipeline for JuliaPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Julia Shader"),
            source: wgpu::ShaderSource::Wgsl(JULIA_SHADER.into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Julia Uniforms"),
            size: std::mem::size_of::<JuliaUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Julia Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Julia Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Julia Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Julia Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
        }
    }
}

const JULIA_SHADER: &str = r#"
struct Uniforms {
    width: f32,
    height: f32,
    c_real: f32,
    c_imag: f32,
    zoom: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full-screen triangle
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );

    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.uv = (positions[vertex_index] + 1.0) / 2.0;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.width / uniforms.height;
    
    // Map UV to complex plane
    var z = vec2<f32>(
        (in.uv.x - 0.5) * 3.0 * aspect / uniforms.zoom,
        (in.uv.y - 0.5) * 3.0 / uniforms.zoom
    );
    
    let c = vec2<f32>(uniforms.c_real, uniforms.c_imag);
    
    var iterations: u32 = 0u;
    let max_iterations: u32 = 256u;
    
    // Julia set iteration: z = z² + c
    for (var i: u32 = 0u; i < max_iterations; i = i + 1u) {
        if (dot(z, z) > 4.0) {
            break;
        }
        z = vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        iterations = i;
    }
    
    if (iterations == max_iterations - 1u) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    
    // Color based on iteration count
    let t = f32(iterations) / f32(max_iterations);
    let color = vec3<f32>(
        0.5 + 0.5 * cos(3.0 + t * 6.28318 + 0.0),
        0.5 + 0.5 * cos(3.0 + t * 6.28318 + 2.094),
        0.5 + 0.5 * cos(3.0 + t * 6.28318 + 4.188)
    );
    
    return vec4<f32>(color, 1.0);
}
"#;
