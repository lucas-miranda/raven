use std::{
    rc::Rc,
    cell::RefCell,
};

use bytemuck::{Pod, Zeroable};
use miette::IntoDiagnostic;

use crate::{
    math::Matrix4x4,
    rendering::{
        shaders::{
            AttributeFormat,
            Bindings,
            BindingsDescriptorEntry,
            Shader,
            ShaderDescriptor,
            ShaderFormat,
            ShaderInfo,
            ShaderInstance,
            ShaderStageKind,
            VertexAttribute, BindingsError, WorldViewProjectionUniforms,
        },
        GraphicAdapter,
        ShaderConfig,
        FrontFace,
        PolygonMode,
        PrimitiveState,
        PrimitiveTopology,
    },
    vertex_attrs,
};

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct Uniforms {
    pub view: Matrix4x4<f32>,
}

impl WorldViewProjectionUniforms for Uniforms {
    fn view(&self) -> &Matrix4x4<f32> {
        &self.view
    }

    fn mut_view(&mut self) -> &mut Matrix4x4<f32> {
        &mut self.view
    }
}

pub struct DefaultShader {
    shader: Shader,
    uniforms: Uniforms,
    default_config: ShaderConfig,
}

impl DefaultShader {
    pub fn new(graphic_adapter: &Rc<RefCell<GraphicAdapter>>) -> Rc<RefCell<Self>> {
        graphic_adapter
            .borrow_mut()
            .shader_builder()
            .create(
                ShaderDescriptor::default()
                    .with_stage(ShaderStageKind::Vertex,    ShaderFormat::WGSL, include_str!("shaders/vert.wgsl"))
                    .with_stage(ShaderStageKind::Fragment,  ShaderFormat::WGSL, include_str!("shaders/frag.wgsl"))
            )
            .set_vertex_attributes(vertex_attrs![
                Float32x2,
                Float32x2,
                Float32x4,
            ].into_iter())
            .bindings(vec![
                BindingsDescriptorEntry::uniform::<Uniforms>(),
            ].into_iter())
            .build()
            .into_diagnostic()
            .unwrap()
    }

    pub fn default_config(&self) -> &ShaderConfig {
        &self.default_config
    }

    pub fn uniforms_mut(&mut self) -> &mut Uniforms {
        &mut self.uniforms
    }
}

impl ShaderInstance for DefaultShader {
    fn new(shader: Shader) -> Self {
        let default_config = ShaderConfig::new(
            &shader,
            PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            }
        );

        Self {
            shader,
            uniforms: Uniforms::default(),
            default_config,
        }
    }

    fn world_view_projection_uniforms(&self) -> Option<&dyn WorldViewProjectionUniforms> {
        Some(&self.uniforms)
    }

    fn mut_world_view_projection_uniforms(&mut self) -> Option<&mut dyn WorldViewProjectionUniforms> {
        Some(&mut self.uniforms)
    }

    fn bindings<'b>(&'b self, mut bindings: Bindings<'b>) -> Result<Bindings<'b>, BindingsError> {
        bindings.uniforms(vec![self.uniforms].as_slice())?;

        Ok(bindings)
    }
}

impl ShaderInfo for DefaultShader {
    fn identifier(&self) -> Shader {
        self.shader
    }
}

impl AsRef<dyn ShaderInstance> for DefaultShader {
    fn as_ref(&self) -> &(dyn ShaderInstance + 'static) {
        self
    }
}
