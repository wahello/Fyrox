use fyrox::core::sstorage::ImmutableString;
use fyrox::renderer::framework::geometry_buffer::{GeometryBuffer, GeometryBufferKind};
use fyrox::renderer::framework::state::{BlendFactor, BlendFunc};
use fyrox::{
    core::{algebra::Matrix4, math::Matrix4Ext},
    renderer::{
        framework::{
            error::FrameworkError,
            framebuffer::DrawParameters,
            gpu_program::{GpuProgram, UniformLocation},
            state::PipelineState,
        },
        RenderPassStatistics, SceneRenderPass, SceneRenderPassContext,
    },
    resource::texture::{CompressionOptions, Texture},
    scene::mesh::surface::SurfaceData,
};
use std::cell::RefCell;
use std::rc::Rc;

struct OverlayShader {
    program: GpuProgram,
    view_projection_matrix: UniformLocation,
    world_matrix: UniformLocation,
    camera_side_vector: UniformLocation,
    camera_up_vector: UniformLocation,
    diffuse_texture: UniformLocation,
    size: UniformLocation,
}

impl OverlayShader {
    pub fn new(state: &mut PipelineState) -> Result<Self, FrameworkError> {
        let fragment_source = include_str!("../resources/embed/shaders/overlay_fs.glsl");
        let vertex_source = include_str!("../resources/embed/shaders/overlay_vs.glsl");
        let program =
            GpuProgram::from_source(state, "OverlayShader", vertex_source, fragment_source)?;
        Ok(Self {
            view_projection_matrix: program
                .uniform_location(state, &ImmutableString::new("viewProjectionMatrix"))?,
            world_matrix: program.uniform_location(state, &ImmutableString::new("worldMatrix"))?,
            camera_side_vector: program
                .uniform_location(state, &ImmutableString::new("cameraSideVector"))?,
            camera_up_vector: program
                .uniform_location(state, &ImmutableString::new("cameraUpVector"))?,
            size: program.uniform_location(state, &ImmutableString::new("size"))?,
            diffuse_texture: program
                .uniform_location(state, &ImmutableString::new("diffuseTexture"))?,
            program,
        })
    }
}

pub struct OverlayRenderPass {
    quad: GeometryBuffer,
    shader: OverlayShader,
    sound_icon: Texture,
    light_icon: Texture,
}

impl OverlayRenderPass {
    pub fn new(state: &mut PipelineState) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            quad: GeometryBuffer::from_surface_data(
                &SurfaceData::make_collapsed_xy_quad(),
                GeometryBufferKind::StaticDraw,
                state,
            ),
            shader: OverlayShader::new(state).unwrap(),
            sound_icon: Texture::load_from_memory(
                include_bytes!("../resources/embed/sound_source.png"),
                CompressionOptions::NoCompression,
                false,
            )
            .unwrap(),
            light_icon: Texture::load_from_memory(
                include_bytes!("../resources/embed/light_source.png"),
                CompressionOptions::NoCompression,
                false,
            )
            .unwrap(),
        }))
    }
}

impl SceneRenderPass for OverlayRenderPass {
    fn on_hdr_render(
        &mut self,
        ctx: SceneRenderPassContext,
    ) -> Result<RenderPassStatistics, FrameworkError> {
        let view_projection = ctx.camera.view_projection_matrix();
        let shader = &self.shader;
        let inv_view = ctx.camera.inv_view_matrix().unwrap();
        let camera_up = -inv_view.up();
        let camera_side = inv_view.side();
        let sound_icon = ctx
            .texture_cache
            .get(ctx.pipeline_state, &self.sound_icon)
            .unwrap();
        let light_icon = ctx
            .texture_cache
            .get(ctx.pipeline_state, &self.light_icon)
            .unwrap();

        for node in ctx.scene.graph.linear_iter() {
            let icon =
                if node.is_directional_light() || node.is_spot_light() || node.is_point_light() {
                    light_icon.clone()
                } else if node.is_sound() {
                    sound_icon.clone()
                } else {
                    continue;
                };

            let position = node.global_position();
            let world_matrix = Matrix4::new_translation(&position);

            ctx.framebuffer.draw(
                &self.quad,
                ctx.pipeline_state,
                ctx.viewport,
                &shader.program,
                &DrawParameters {
                    cull_face: None,
                    color_write: Default::default(),
                    depth_write: false,
                    stencil_test: None,
                    depth_test: true,
                    blend: Some(BlendFunc {
                        sfactor: BlendFactor::SrcAlpha,
                        dfactor: BlendFactor::OneMinusSrcAlpha,
                    }),
                    stencil_op: Default::default(),
                },
                |mut program_binding| {
                    program_binding
                        .set_matrix4(&shader.view_projection_matrix, &view_projection)
                        .set_matrix4(&shader.world_matrix, &world_matrix)
                        .set_vector3(&shader.camera_side_vector, &camera_side)
                        .set_vector3(&shader.camera_up_vector, &camera_up)
                        .set_f32(&shader.size, 0.33)
                        .set_texture(&shader.diffuse_texture, &icon);
                },
            );
        }

        Ok(Default::default())
    }
}
