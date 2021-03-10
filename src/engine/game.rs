use cgmath::Vector3;
use winit::window::Window;

use crate::engine::entity::Entity;
use crate::engine::scene::Scene;
use crate::renderer::context::Context;
use crate::renderer::datatypes::Vertex;

pub struct VulkrapApplication {
    context: Context,
    scene: Scene,

    elapsed_time_s: f32,
}

impl VulkrapApplication {
    pub fn new(window: &Window) -> VulkrapApplication {
        let mut context = Context::new(window);

        // TODO
        context.add_pipeline();

        let mut scene = Scene::new();

        for entity in _create_static_entities() {
            scene.add_entity(&mut context, entity);
        }

        VulkrapApplication {
            context,
            scene,
            elapsed_time_s: 0.0,
        }
    }

    pub fn update(&mut self, delta_time_s: f32) {
        self.elapsed_time_s += delta_time_s;

        let render_job = self.scene.get_render_job();
        self.context.draw_frame(self.elapsed_time_s, render_job);
    }

    pub fn exit(&self) {
        unsafe {
            self.context.wait_idle();
        }
    }
}

fn _create_static_entities() -> Vec<Entity> {
    let triangle3 = vec![
        Vertex::new(Vector3::new(-0.5, -0.5, -1.0), Vector3::new(1.0, 0.0, 0.0)),
        Vertex::new(Vector3::new(0.5, -0.5, -1.0), Vector3::new(0.0, 1.0, 0.0)),
        Vertex::new(Vector3::new(-0.5, 0.5, -1.0), Vector3::new(0.0, 0.0, 1.0)),
        Vertex::new(Vector3::new(0.5, 0.5, -1.0), Vector3::new(1.0, 0.0, 1.0)),
    ];
    let indices3 = vec![0, 1, 2, 2, 1, 3];
    let crazy_quad = Entity::new(triangle3, indices3);

    let triangle = vec![
        Vertex::new(Vector3::new(-1.0, -0.25, -2.0), Vector3::new(1.0, 0.0, 0.0)),
        Vertex::new(Vector3::new(0.0, -0.25, -2.0), Vector3::new(0.0, 1.0, 0.0)),
        Vertex::new(Vector3::new(-1.0, 0.25, -2.0), Vector3::new(0.0, 0.0, 1.0)),
        Vertex::new(Vector3::new(0.0, 0.25, -2.0), Vector3::new(1.0, 0.0, 1.0)),
    ];
    let indices = vec![0, 1, 2, 2, 1, 3];
    let entity = Entity::new(triangle, indices);

    let triangle2 = vec![
        Vertex::new(Vector3::new(0.5, -0.25, -2.0), Vector3::new(1.0, 0.0, 0.0)),
        Vertex::new(Vector3::new(1.5, -0.25, -2.0), Vector3::new(1.0, 1.0, 0.0)),
        Vertex::new(Vector3::new(0.5, 0.25, -2.0), Vector3::new(1.0, 0.0, 1.0)),
        Vertex::new(Vector3::new(1.5, 0.25, -2.0), Vector3::new(1.0, 0.0, 1.0)),
    ];
    let indices2 = vec![0, 1, 2, 2, 1, 3];
    let entity2 = Entity::new(triangle2, indices2);

    let mut entities = Vec::with_capacity(10);
    entities.push(crazy_quad);
    entities.push(entity);
    entities.push(entity2);

    entities
}
