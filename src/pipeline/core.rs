use crate::backend::{WindowBackend, ComponentRenderer, RawOSEvent, PropertyValue, ViewKitBackend};
use std::any::Any;

pub struct BackendImpl {
    width: u32,
    height: u32,
}

impl BackendImpl {
    pub fn new() -> Result<Self, String> {
        println!("ViewKit: Connecting to Wayland display...");
        Ok(Self { width: 0, height: 0 })
    }
}

impl WindowBackend for BackendImpl {
    fn create_window(&mut self, width: u32, height: u32, title: &str, no_decoration: bool) {
        self.width = width;
        self.height = height;
        println!("ViewKit: xdg_shell :: Create window '{}' ({}x{}), deco: {}", title, width, height, !no_decoration);
    }
    fn swap_buffers(&mut self, _buffer: &[u32], _width: u32, _height: u32) {}
    fn poll_os_event(&mut self) -> Option<RawOSEvent> { None }
    fn as_any(&self) -> &dyn Any { self }
}

impl ComponentRenderer for BackendImpl {
    fn register_component(&mut self, name: &str, _template_html: &str) -> Result<(), String> {
        println!("ViewKit: Registered HTML component '{}'", name);
        Ok(())
    }
    fn update_ui_tree(&mut self, tree_delta_json: &str) {
        println!("ViewKit: Updating UI Tree -> {}", tree_delta_json);
    }
    fn set_component_property(&mut self, _component_id: &str, _key: &str, _value: PropertyValue) {}
}

impl ViewKitBackend for BackendImpl {}