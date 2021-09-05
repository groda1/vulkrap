use std::borrow::BorrowMut;
use std::collections::HashMap;

pub const M_SENSITIVITY: u32 = 1000;
pub const M_YAW: u32 = 1001;
pub const M_PITCH: u32 = 1002;

pub struct ConfigVariables {
    id_to_cvar: HashMap<u32, ConfigVariable>,
    cvar_str_to_id: HashMap<String, u32>,
}

impl ConfigVariables {
    pub fn new() -> Self {
        let mut id_to_cvar = HashMap::new();

        id_to_cvar.insert(
            M_SENSITIVITY,
            ConfigVariable::new("m_sensitivity", 0.08f32, "Mouse sensitivity"),
        );
        id_to_cvar.insert(M_YAW, ConfigVariable::new("m_yaw", 0.01f32, "Mouse yaw"));
        id_to_cvar.insert(M_PITCH, ConfigVariable::new("m_pitch", 0.01f32, "Mouse pitch"));

        let cvar_str_to_id = id_to_cvar
            .iter()
            .map(|(id, cvar)| (cvar.name.to_string(), *id))
            .collect();

        ConfigVariables {
            id_to_cvar,
            cvar_str_to_id,
        }
    }

    pub fn get(&self, id: u32) -> &dyn CvarValue {
        let cvar = self.id_to_cvar.get(&id);
        cvar.unwrap().value.as_ref()
    }

    pub fn set<T: CvarValue>(&mut self, id: u32, val: T) {
        let cvar = self.id_to_cvar.get_mut(&id).unwrap();
        cvar.value.set(&val);
    }
}

struct ConfigVariable {
    name: &'static str,
    value: Box<dyn CvarValue>,
    default: Box<dyn CvarValue>,
    description: &'static str,
}

impl ConfigVariable {
    pub fn new<T: CvarValue + Copy + 'static>(name: &'static str, default: T, description: &'static str) -> Self {
        ConfigVariable {
            name,
            value: Box::new(default),
            default: Box::new(default),
            description,
        }
    }
}

pub trait CvarValue {
    fn get_float(&self) -> f32;
    fn get_int(&self) -> u32;

    fn set(&mut self, val: &dyn CvarValue);
}

impl CvarValue for f32 {
    fn get_float(&self) -> f32 {
        *self
    }

    fn get_int(&self) -> u32 {
        *self as u32
    }

    fn set(&mut self, val: &dyn CvarValue) {
        *self = val.get_float();
    }
}

impl CvarValue for u32 {
    fn get_float(&self) -> f32 {
        *self as f32
    }

    fn get_int(&self) -> u32 {
        *self
    }

    fn set(&mut self, val: &dyn CvarValue) {
        *self = val.get_int();
    }
}
