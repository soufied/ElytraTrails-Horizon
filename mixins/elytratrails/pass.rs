use super::WorldDrawn;
use super::elytratrails as et;
use glam::{DVec3, Vec4};
use std::sync::OnceLock;
use std::time::Instant;
use wgpu::util::DeviceExt;

const SHADER: &str = "
struct Vertex {
    @location(0) clip: vec4<f32>,
    @location(1) color: vec4<f32>,
}

struct Varyings {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_trail(v: Vertex) -> Varyings {
    var out: Varyings;
    out.clip = v.clip;
    out.color = v.color;
    return out;
}

@fragment
fn fs_trail(i: Varyings) -> @location(0) vec4<f32> {
    return i.color;
}
";

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    clip: [f32; 4],
    color: [f32; 4],
}

static PIPELINE: OnceLock<(wgpu::TextureFormat, wgpu::RenderPipeline)> = OnceLock::new();

fn pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> Option<&'static wgpu::RenderPipeline> {
    let (stored, pipeline) = PIPELINE.get_or_init(|| (format, build(device, format)));
    (*stored == format).then_some(pipeline)
}

fn build(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ElytraTrails shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("ElytraTrails layout"),
        bind_group_layouts: &[],
        ..Default::default()
    });
    let attributes = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ElytraTrails"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_trail"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &attributes,
            })],
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_trail"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn clip(world: &WorldDrawn<'_>, point: [f64; 3]) -> [f32; 4] {
    let relative = DVec3::from_array(point) - world.camera;
    let v = world.jittered
        * Vec4::new(
            relative.x as f32,
            relative.y as f32,
            relative.z as f32,
            1.0,
        );
    v.to_array()
}

fn rgba(tint: u32, alpha: f32) -> [f32; 4] {
    [
        ((tint >> 16) & 0xFF) as f32 / 255.0,
        ((tint >> 8) & 0xFF) as f32 / 255.0,
        (tint & 0xFF) as f32 / 255.0,
        alpha,
    ]
}

pub(crate) fn draw(world: &mut WorldDrawn<'_>) {
    if world.depth_size != world.size {
        return;
    }
    let ribbons = et::ribbons(Instant::now(), et::current_options());
    if ribbons.is_empty() {
        return;
    }
    let Some(pipeline) = pipeline(world.device, world.color_format) else {
        return;
    };
    let mut vertices = Vec::with_capacity(ribbons.len() * 12);
    for ribbon in &ribbons {
        let color = rgba(ribbon.tint, ribbon.alpha);
        for quad in et::sheets(ribbon.a, ribbon.b, ribbon.width) {
            for index in [0, 1, 2, 0, 2, 3] {
                vertices.push(Vertex {
                    clip: clip(world, quad[index]),
                    color,
                });
            }
        }
    }
    let buffer = world.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ElytraTrails vertices"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    {
        let mut pass = world.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ElytraTrails"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: world.color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: world.depth,
                depth_ops: None,
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_pipeline(pipeline);
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }
}
