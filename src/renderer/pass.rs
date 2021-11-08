use std::collections::HashMap;

pub type RenderPassHandle = u32;

struct RenderPass {
    pass_order: u32,
}




struct RenderPassHandler {
    render_passes: HashMap<RenderPassHandle, RenderPass>,
    pass_order: Vec<RenderPassHandle>,

}

impl RenderPassHandler {

    pub fn new() -> Self {
        Self {
            render_passes: HashMap::new(),
            pass_order: Vec::new(),
        }
    }

    pub fn add(&mut self, render_pass : RenderPass) -> Result<RenderPassHandle, &str> {

        if self.render_passes.contains_key(&render_pass.pass_order) {
            return Err("a render pass with same order already exists!");
        }

        let handle = render_pass.pass_order;
        self.render_passes.insert(handle, render_pass);

        self.pass_order.push(handle);
        self.pass_order.sort();

        Ok(handle)
    }



    pub fn derp(&mut self) {


    }



}