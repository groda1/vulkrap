use std::collections::HashMap;

pub const WINDOW_TITLE: u32 = 100;
pub const WINDOW_WIDTH: u32 = 101;
pub const WINDOW_HEIGHT: u32 = 102;

pub const M_SENSITIVITY: u32 = 1000;
pub const M_YAW: u32 = 1001;
pub const M_PITCH: u32 = 1002;

pub const FOV: u32 = 1050;

pub const TEST: u32 = 5000;

pub struct ConfigVariables {
    id_to_cvar: HashMap<u32, ConfigVariable>,
    cvar_str_to_id: HashMap<String, u32>,

    dirty: bool,
}

impl ConfigVariables {
    pub fn new() -> Self {
        let mut id_to_cvar = HashMap::new();

        id_to_cvar.insert(WINDOW_TITLE, ConfigVariable::new("window_title", "Untitled".to_string(), "Window title"));
        id_to_cvar.insert(WINDOW_WIDTH, ConfigVariable::new("window_width", 1920, "Window title"));
        id_to_cvar.insert(WINDOW_HEIGHT, ConfigVariable::new("window_height", 1080, "Window title"));

        id_to_cvar.insert(
            M_SENSITIVITY,
            ConfigVariable::new("m_sensitivity", 0.08f32, "Mouse sensitivity"),
        );
        id_to_cvar.insert(M_YAW, ConfigVariable::new("m_yaw", 0.01f32, "Mouse yaw"));
        id_to_cvar.insert(M_PITCH, ConfigVariable::new("m_pitch", 0.01f32, "Mouse pitch"));

        id_to_cvar.insert(FOV, ConfigVariable::new("fov", 60.0f32, "Vertical field of view"));

        id_to_cvar.insert(TEST, ConfigVariable::new("test", String::from("pepsi!"), "Test1"));

        let cvar_str_to_id = id_to_cvar
            .iter()
            .map(|(id, cvar)| (cvar.name.to_string(), *id))
            .collect();

        ConfigVariables {
            id_to_cvar,
            cvar_str_to_id,
            dirty: false,
        }
    }

    pub fn get(&self, id: u32) -> &dyn CvarValue {
        let cvar = self.id_to_cvar.get(&id);
        cvar.unwrap().value.as_ref()
    }

    pub fn get_desc(&self, id: u32) -> String {
        let cvar = self.id_to_cvar.get(&id).expect("unknown cvar id");
        match cvar.value.get_type() {
            CvarType::Float => {
                format!(
                    "{} = {} ({}, type: float, default: {})",
                    cvar.name,
                    cvar.value.as_float(),
                    cvar.description,
                    cvar.default.as_float()
                )
            }
            CvarType::Integer => {
                format!(
                    "{} = {} ({}, type: int, default: {})",
                    cvar.name,
                    cvar.value.as_int(),
                    cvar.description,
                    cvar.default.as_int()
                )
            }
            CvarType::String => {
                format!(
                    "{} = \"{}\" ({}, type: str, default: \"{}\")",
                    cvar.name,
                    cvar.value.as_str(),
                    cvar.description,
                    cvar.default.as_str()
                )
            }
        }
    }

    pub fn get_cvar_id_from_str(&self, cvar_str: &str) -> Option<u32> {
        self.cvar_str_to_id.get(cvar_str).copied()
    }

    pub fn set<T: CvarValue>(&mut self, id: u32, val: T) {
        let cvar = self.id_to_cvar.get_mut(&id).unwrap();
        cvar.value.set(&val);
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

struct ConfigVariable {
    name: &'static str,
    value: Box<dyn CvarValue>,
    default: Box<dyn CvarValue>,
    description: &'static str,
}

impl ConfigVariable {
    pub fn new<T: CvarValue + Clone + 'static>(name: &'static str, default: T, description: &'static str) -> Self {
        ConfigVariable {
            name,
            value: Box::new(default.clone()),
            default: Box::new(default),
            description,
        }
    }
}

#[derive(Debug)]
pub enum CvarType {
    Float,
    Integer,
    String,
}

pub trait CvarValue {
    fn as_float(&self) -> f32;
    fn as_int(&self) -> u32;
    fn as_str(&self) -> String;
    fn get_type(&self) -> CvarType;
    fn set(&mut self, val: &dyn CvarValue);
}

impl CvarValue for f32 {
    fn as_float(&self) -> f32 {
        *self
    }

    fn as_int(&self) -> u32 {
        *self as u32
    }

    fn as_str(&self) -> String {
        format!("{}", *self)
    }

    fn get_type(&self) -> CvarType {
        CvarType::Float
    }

    fn set(&mut self, val: &dyn CvarValue) {
        *self = val.as_float();
    }
}

impl CvarValue for u32 {
    fn as_float(&self) -> f32 {
        *self as f32
    }

    fn as_int(&self) -> u32 {
        *self
    }

    fn as_str(&self) -> String {
        format!("{}", *self)
    }

    fn get_type(&self) -> CvarType {
        CvarType::Integer
    }

    fn set(&mut self, val: &dyn CvarValue) {
        *self = val.as_int();
    }
}

impl CvarValue for String {
    fn as_float(&self) -> f32 {
        panic!()
    }

    fn as_int(&self) -> u32 {
        panic!()
    }

    fn as_str(&self) -> String {
        self.clone()
    }

    fn get_type(&self) -> CvarType {
        CvarType::String
    }

    fn set(&mut self, val: &dyn CvarValue) {
        *self = val.as_str();
    }
}
