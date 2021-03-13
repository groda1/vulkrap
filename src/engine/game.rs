use cgmath::{Deg, Matrix4, One, Point3, Quaternion, Rotation3, Vector3};
use winit::window::Window;

use crate::engine::datatypes::{ColoredVertex, ViewProjectionUniform};
use crate::engine::entity::Entity;
use crate::engine::mesh::{MeshManager, PredefinedMesh};
use crate::engine::scene::Scene;
use crate::renderer::context::Context;
use crate::renderer::pipeline::{PipelineConfiguration, PipelineHandle};
use crate::util::file;
use std::path::Path;

pub struct VulkrapApplication {
    context: Context,
    mesh_manager: MeshManager,
    scene: Scene,

    main_pipeline: PipelineHandle,

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
            .build();

        let main_pipeline = context.add_pipeline::<ColoredVertex>(pipeline_config);

        let mut scene = Scene::new(main_pipeline);

        for entity in create_entities(&mesh_manager) {
            scene.add_entity(&mut context, entity);
        }

        VulkrapApplication {
            context,
            mesh_manager,
            scene,
            main_pipeline,
            elapsed_time_s: 0.0,
        }
    }

    pub fn update(&mut self, delta_time_s: f32) {
        self.elapsed_time_s += delta_time_s;

        self.scene.update(delta_time_s, self.elapsed_time_s);
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
        let wobble = self.elapsed_time_s * 5.0;

        let data = ViewProjectionUniform {
            view: Matrix4::look_at_rh(
                Point3::new(0.0, -0.1, 2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
            ),
            proj: cgmath::perspective(Deg(45.0), self.context.get_aspect_ratio(), 0.1, 10.0),
            wobble,
        };
        self.context.update_pipeline_uniform_data(self.main_pipeline, data);
    }
}

fn create_entities(mesh_manager: &MeshManager) -> Vec<Entity> {
    let quad1 = Entity::new(
        Vector3::new(0.0, 0.0, -1.0),
        Quaternion::from_angle_z(Deg(0.0)),
        *mesh_manager.get_predefined_mesh(PredefinedMesh::QUAD),
    );

    // let triangle3 = vec![
    //     ColoredVertex::new(Vector3::new(-0.5, -0.5, -1.0), Vector3::new(1.0, 0.0, 0.0)),
    //     ColoredVertex::new(Vector3::new(0.5, -0.5, -1.0), Vector3::new(0.0, 1.0, 0.0)),
    //     ColoredVertex::new(Vector3::new(-0.5, 0.5, -1.0), Vector3::new(0.0, 0.0, 1.0)),
    //     ColoredVertex::new(Vector3::new(0.5, 0.5, -1.0), Vector3::new(1.0, 0.0, 1.0)),
    // ];
    // let indices3 = vec![0, 1, 2, 2, 1, 3];
    // let crazy_quad = Entity::new(triangle3, indices3);
    //
    // let triangle = vec![
    //     ColoredVertex::new(Vector3::new(-1.0, -0.25, -2.0), Vector3::new(1.0, 0.0, 0.0)),
    //     ColoredVertex::new(Vector3::new(0.0, -0.25, -2.0), Vector3::new(0.0, 1.0, 0.0)),
    //     ColoredVertex::new(Vector3::new(-1.0, 0.25, -2.0), Vector3::new(0.0, 0.0, 1.0)),
    //     ColoredVertex::new(Vector3::new(0.0, 0.25, -2.0), Vector3::new(1.0, 0.0, 1.0)),
    // ];
    // let indices = vec![0, 1, 2, 2, 1, 3];
    // let entity = Entity::new(triangle, indices);
    //
    // let triangle2 = vec![
    //     ColoredVertex::new(Vector3::new(0.5, -0.25, -2.0), Vector3::new(1.0, 0.0, 0.0)),
    //     ColoredVertex::new(Vector3::new(1.5, -0.25, -2.0), Vector3::new(1.0, 1.0, 0.0)),
    //     ColoredVertex::new(Vector3::new(0.5, 0.25, -2.0), Vector3::new(1.0, 0.0, 1.0)),
    //     ColoredVertex::new(Vector3::new(1.5, 0.25, -2.0), Vector3::new(1.0, 0.0, 1.0)),
    // ];
    // let indices2 = vec![0, 1, 2, 2, 1, 3];
    // let entity2 = Entity::new(triangle2, indices2);
    //
    let mut entities = Vec::with_capacity(10);
    entities.push(quad1);
    // entities.push(entity);
    // entities.push(entity2);

    entities
}
