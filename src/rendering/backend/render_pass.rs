use wgpu::util::DeviceExt;

use crate::{
    rendering::{
        shaders::{
            Shader,
            builder::{ShaderBuilder, ShaderContext}, ShaderInstance
        },
        Color,
        DrawConfig,
    },
    math::Vector2,
};

pub struct RenderPass<'a> {
    encoder: wgpu::CommandEncoder,
    queue: &'a wgpu::Queue,
    device: &'a wgpu::Device,
    surface_view: &'a wgpu::TextureView,
    vertex_data: Vec<Vector2<f32>>,
    bind_group: Option<wgpu::BindGroup>,
    shader_context: &'a ShaderContext,
    config: &'a DrawConfig,
    clear_color: Option<Color<f32>>,
}

impl<'a> RenderPass<'a> {
    pub(super) fn new(
        encoder: wgpu::CommandEncoder,
        queue: &'a wgpu::Queue,
        surface_view: &'a wgpu::TextureView,
        device: &'a wgpu::Device,
        bind_group: Option<wgpu::BindGroup>,
        shader_context: &'a ShaderContext,
        config: &'a DrawConfig,
    ) -> Self {
        Self {
            encoder,
            queue,
            device,
            surface_view,
            vertex_data: Vec::new(),
            bind_group,
            shader_context,
            config,
            clear_color: None,
        }
    }

    pub fn clear_color<C: Into<Color<f32>>>(mut self, color: C) -> Self {
        self.clear_color = Some(color.into());

        self
    }

    pub fn extend_vertices<T: IntoIterator<Item = Vector2<f32>>>(mut self, iter: T) -> Self {
        self.vertex_data.extend(iter);

        self
    }

    //pub fn using_shader<U: bytemuck::Zeroable + bytemuck::Pod + bytemuck::NoUninit>(mut self, shader: &'a Shader, uniforms: Option<&U>) -> Self {
    /*
    pub fn using_shader(mut self, shader: &'a S) -> Self {
        self.shader = Some(shader);

        self.bind_group = {
            let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("uniforms buffer"),
                contents: bytemuck::cast_slice(&[*shader.uniforms()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let shader_context = self.shader_builder
                .get_context(&shader.id())
                .unwrap();

            let bind_group = Some(
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Uniform Bind Group"),
                    layout: &shader_context.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: uniform_buffer.as_entire_binding(),
                        }
                    ],
                })
            );

            bind_group
        };

        self
    }
    */

    pub fn submit(mut self) -> Result<(), super::RenderBackendOperationError> {
        if !self.vertex_data.is_empty() {
            self.vertex_data
                .iter_mut()
                .for_each(|v| *v += self.config.position);
        }

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(self.vertex_data.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // create wgpu render pass and submit

        {
            let load = match self.clear_color {
                Some(clear_color) => wgpu::LoadOp::Clear(clear_color.into()),
                None => wgpu::LoadOp::Load,
            };

            let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.shader_context.pipeline);

            if let Some(ref bindings) = self.bind_group {
                pass.set_bind_group(0, bindings, &[]);
            }

            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..(self.vertex_data.len() as u32), 0..1);
        }

        // TODO  try to submit multiple command buffers at once?
        self.queue.submit(Some(self.encoder.finish()));

        Ok(())
    }
}
