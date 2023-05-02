use std::{
    rc::{Rc, Weak},
    cell::RefCell
};

use bytemuck::{Pod, Zeroable};

use crate::{
    components::{
        transform::Transform,
        GraphicDisplayer,
    },
    ecs::{
        component::{
            self,
            BaseQuery,
            QueryEntry,
        },
        system::System,
    },
    input,
    math::Matrix4x4,
    rendering::{
        shaders::{
            builder::{
                PrimitiveTopology,
                ShaderFormat,
            },
            AttributeFormat,
            Shader,
            ShaderId,
            ShaderInstance,
            ShaderUniformInstance,
            VertexAttribute,
        },
        Color,
        DrawBatcher,
        DrawConfig,
        GraphicAdapter,
    },
    vertex_attrs,
};

//

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
struct MyUniforms {
    pub view: Matrix4x4<f32>,
    pub color: Color<f32>,
}

struct MyShader {
    shader: Shader,
    uniforms: Vec<MyUniforms>,
}

impl MyShader {
}

impl ShaderInstance for MyShader {
    fn new(shader: Shader) -> Self {
        Self {
            shader,
            uniforms: vec![MyUniforms::default()],
        }
    }

    fn id(&self) -> ShaderId {
        self.shader.id()
    }

    fn uniforms_as_slice<'s>(&'s self) -> &'s [u8] {
        bytemuck::cast_slice(self.uniforms.as_slice())
    }
}

impl ShaderUniformInstance for MyShader {
    type Uniforms = MyUniforms;

    fn uniforms(&self) -> &Self::Uniforms {
        self.uniforms.get(0).unwrap()
    }
}

impl AsRef<dyn ShaderInstance> for MyShader {
    fn as_ref(&self) -> &(dyn ShaderInstance + 'static) {
        self
    }
}

//

struct MyShader2 {
    shader: Shader,
    uniforms: Vec<MyUniforms>,
}

impl MyShader2 {
}

impl ShaderInstance for MyShader2 {
    fn new(shader: Shader) -> Self {
        Self {
            shader,
            uniforms: vec![MyUniforms::default()],
        }
    }

    fn id(&self) -> ShaderId {
        self.shader.id()
    }

    fn uniforms_as_slice<'s>(&'s self) -> &'s [u8] {
        bytemuck::cast_slice(self.uniforms.as_slice())
    }
}

impl ShaderUniformInstance for MyShader2 {
    type Uniforms = MyUniforms;

    fn uniforms(&self) -> &Self::Uniforms {
        self.uniforms.get(0).unwrap()
    }
}

impl AsRef<dyn ShaderInstance> for MyShader2 {
    fn as_ref(&self) -> &(dyn ShaderInstance + 'static) {
        self
    }
}

//

//

pub struct RenderSystem {
    graphic_adapter: Weak<RefCell<GraphicAdapter>>,
    shader: MyShader,
    shader2: MyShader2,
}

impl RenderSystem {
    pub fn new(graphic_adapter: &Rc<RefCell<GraphicAdapter>>) -> Self {
        let shader = graphic_adapter
            .borrow_mut()
            .shader_builder()
            .create::<MyUniforms>(
                ShaderFormat::GLSL,
                include_str!("shaders/p1.vert"),
                include_str!("shaders/p1.frag"),
                PrimitiveTopology::TriangleList,
            )
            .set_vertex_attributes(vertex_attrs![
                Float32x2,
            ].into_iter())
            .build();

        let shader2 = graphic_adapter
            .borrow_mut()
            .shader_builder()
            .create::<MyUniforms>(
                ShaderFormat::GLSL,
                include_str!("shaders/p1.vert"),
                include_str!("shaders/p1.frag"),
                PrimitiveTopology::LineList,
            )
            .set_vertex_attributes(vertex_attrs![
                Float32x2,
            ].into_iter())
            .build();

        Self {
            graphic_adapter: Rc::downgrade(graphic_adapter),
            shader,
            shader2,
        }
    }
}

impl System for RenderSystem {
    type Query<'q> = (
        component::Query<'q, GraphicDisplayer>,
        component::Query<'q, Transform>,
    );

    fn setup(&mut self) {
    }

    fn input<'q>(&mut self, _query: Self::Query<'q>, _event: &input::DeviceEvent) {
    }

    fn run<'q>(&mut self, query: Self::Query<'q>) {
        println!(
            "[RenderSystem] captured components({}): {} GraphicDisplayer, {} Transform",
            query.iter_components().count(),
            query.0.iter_components().count(),
            query.1.iter_components().count()
        );

        let graphic_adapter = self.graphic_adapter.upgrade().unwrap();


        {
            let mut uniforms = self.shader.uniforms.get_mut(0).unwrap();
            uniforms.view = Matrix4x4::ortho(0.0, 180.0, 0.0, 320.0, -100.0, 100.0);
            uniforms.color = Color::rgba(1.0, 0.0, 1.0, 1.0);
        }

        {
            let mut uniforms = self.shader2.uniforms.get_mut(0).unwrap();
            uniforms.view = Matrix4x4::ortho(0.0, 180.0, 0.0, 320.0, -100.0, 100.0);
            uniforms.color = Color::rgba(1.0, 0.0, 1.0, 1.0);
        }

        let mut adapter = graphic_adapter.borrow_mut();

        match adapter.prepare_draw() {
            Ok(mut draw_command) => {
                // TODO  use default shader to clear screen
                draw_command.clear(Color::<u8>::rgb(70, 35, 110), &self.shader)
                            .unwrap();

                // collects everything indo a batcher
                let mut draw_batcher = DrawBatcher::default();
                draw_batcher.register_shader(&self.shader);
                draw_batcher.register_shader(&self.shader2);

                for QueryEntry { component: (a, b), .. } in query.iter_components() {
                    if let Some(graphic_displayer) = a {
                        if let Some(transform) = b {
                            if let Some(ref g) = graphic_displayer.graphic {
                                let draw_config = DrawConfig {
                                    position: transform.position(),
                                    shader_id: 0,
                                };

                                println!("[RenderSystem] Rendering with {:?}", draw_config);
                                println!("[RenderSystem] Transform: {:?}", *transform);

                                // TODO  make graphic be able to have a Shader defined (which will
                                // be forwarded to batcher)
                                g.draw(&mut draw_batcher, draw_config)
                            }
                        }
                    }
                }

                // TODO  make DrawBatcher dispatchs batches itself
                draw_batcher.flush(&mut draw_command);

                /*
                {
                    // make a pass from batched vertices
                    let mut pass = draw_command.begin(&self.shader, None);
                    pass.extend(vertices.collect::<Vec<_>>().iter(), DrawConfig::EMPTY);
                    pass.submit().unwrap();
                }
                */

                draw_command.present();
            },
            Err(_e) => return,
        }
    }

    fn create_query<'q>(&self) -> Self::Query<'q> {
        Self::Query::default()
    }
}
