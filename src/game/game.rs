use cgmath::Vector3;
use winit::window::Window;

use crate::renderer::context::Context;
use crate::renderer::datatypes::Vertex;
use crate::renderer::entity::Entity;

pub struct VulkrapApplication {
    context: Context,
    entities: Vec<Entity>,

    elapsed_time_s: f32,
}

impl VulkrapApplication {
    pub fn new(window: &Window) -> VulkrapApplication {
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

        let mut context = Context::new(window);
        context.add_entity(entity);
        context.add_entity(entity2);

        VulkrapApplication {
            context,
            entities: Vec::new(),
            elapsed_time_s: 0.0,
        }
    }

    pub fn update(&mut self, delta_time_s: f32) {
        self.elapsed_time_s += delta_time_s;
        self.context.draw_frame(self.elapsed_time_s);
    }

    pub fn exit(&self) {
        unsafe {
            self.context.wait_idle();
        }
    }
}
