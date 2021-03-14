use std::path::Path;

use cgmath::{Deg, Matrix4, Point3, Quaternion, Rotation3, Vector3};
use winit::window::Window;

use crate::engine::datatypes::{
    ColoredVertex, SimpleVertex, ViewProjectionUniform, MODEL_COLOR_PUSH_CONSTANT_SIZE, MODEL_WOBLY_PUSH_CONSTANT_SIZE,
};
use crate::engine::entity::{FlatColorEntity, WobblyEntity};
use crate::engine::mesh::{MeshManager, PredefinedMesh};
use crate::engine::scene::Scene;
use crate::renderer::context::Context;
use crate::renderer::pipeline::{PipelineConfiguration, PipelineHandle};
use crate::util::file;

pub struct VulkrapApplication {
    context: Context,
    mesh_manager: MeshManager,
    scene: Scene,

    main_pipeline: PipelineHandle,
    flat_color_pipeline: PipelineHandle,

    elapsed_time_s: f32,
}

impl VulkrapApplication {
    pub fn new(window: &Window) -> VulkrapApplication {
        let mut context = Context::new(window);
        let mesh_manager = MeshManager::new(&mut context);

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new(
                "./resources/shaders/crazy_triangle_vert.spv",
            )))
            .with_fragment_shader(file::read_file(Path::new(
                "./resources/shaders/crazy_triangle_frag.spv",
            )))
            .with_push_constant(MODEL_WOBLY_PUSH_CONSTANT_SIZE)
            .build();
        let main_pipeline = context.add_pipeline::<ColoredVertex>(pipeline_config);

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/flat_color_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/flat_color_frag.spv")))
            .with_push_constant(MODEL_COLOR_PUSH_CONSTANT_SIZE)
            .build();
        let flat_color_pipeline = context.add_pipeline::<SimpleVertex>(pipeline_config);

        let scene = Scene::new(main_pipeline, flat_color_pipeline);

        let mut app = VulkrapApplication {
            context,
            mesh_manager,
            scene,
            main_pipeline,
            flat_color_pipeline,
            elapsed_time_s: 0.0,
        };
        app.create_entities();

        app
    }

    pub fn update(&mut self, delta_time_s: f32) {
        self.elapsed_time_s += delta_time_s;

        self.scene.update(delta_time_s);
        self.update_uniform_data(delta_time_s);

        let render_job = self.scene.get_render_job();
        self.context.draw_frame(render_job);
    }

    pub fn exit(&self) {
        unsafe {
            self.context.wait_idle();
        }
    }

    fn update_uniform_data(&mut self, _delta_time_s: f32) {
        let data = ViewProjectionUniform {
            view: Matrix4::look_at_rh(
                Point3::new(0.0, -0.1, -2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
            ),
            proj: cgmath::perspective(Deg(45.0), self.context.get_aspect_ratio(), 0.1, 10.0),
        };

        // TODO LOOP
        self.context.update_pipeline_uniform_data(self.main_pipeline, data);
        self.context
            .update_pipeline_uniform_data(self.flat_color_pipeline, data);
    }

    fn create_entities(&mut self) {
        let quad1 = WobblyEntity::new(
            Vector3::new(0.0, 0.0, 1.0),
            Quaternion::from_angle_z(Deg(0.0)),
            *self.mesh_manager.get_predefined_mesh(PredefinedMesh::ColoredQuad),
            0.0,
        );

        let quad2 = WobblyEntity::new(
            Vector3::new(0.5, 1.0, 2.0),
            Quaternion::from_angle_z(Deg(0.0)),
            *self.mesh_manager.get_predefined_mesh(PredefinedMesh::ColoredTriangle),
            0.0,
        );

        let triangle = FlatColorEntity::new(
            Vector3::new(-0.5, 1.0, 2.0),
            Quaternion::from_angle_z(Deg(37.0)),
            *self.mesh_manager.get_predefined_mesh(PredefinedMesh::SimpleTriangle),
            Vector3::new(1.0, 0.0, 0.0),
        );

        self.scene.add_wobbly_entity(quad1);
        self.scene.add_wobbly_entity(quad2);
        self.scene.add_flat_color_entity(triangle);
    }
}
