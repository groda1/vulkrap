use std::collections::HashMap;
use crate::engine::runtime::ControlSignal;
use crate::{MIN_HEIGHT, MIN_WIDTH};

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

        id_to_cvar.insert(WINDOW_TITLE, ConfigVariable::builder()
            .name("window_title")
            .default("Untitled".to_string())
            .description("Window title")
            .build());
        id_to_cvar.insert(WINDOW_WIDTH, ConfigVariable::builder()
            .name("window_width")
            .default(1920)
            .min_value(MIN_WIDTH)
            .description("Window width")
            .change_trigger(ControlSignal::ResizeWindow)
            .build());
        id_to_cvar.insert(WINDOW_HEIGHT, ConfigVariable::builder()
            .name("window_height")
            .default(1080)
            .min_value(MIN_HEIGHT)
            .description("Window height")
            .change_trigger(ControlSignal::ResizeWindow)
            .build());

        id_to_cvar.insert(M_SENSITIVITY, ConfigVariable::builder()
            .name("m_sensitivity")
            .default(0.08f32)
            .description("Mouse sensitivity")
            .build());
        id_to_cvar.insert(M_YAW, ConfigVariable::builder()
            .name("m_yaw")
            .default(0.01f32)
            .description("Mouse yaw")
            .build());
        id_to_cvar.insert(M_PITCH, ConfigVariable::builder()
            .name("m_pitch")
            .default(0.01f32)
            .description("Mouse pitch")
            .build());

        id_to_cvar.insert(FOV, ConfigVariable::builder()
            .name("fov")
            .default( 60f32)
            .min_value(10f32)
            .max_value(170f32)
            .description( "Vertical field of view")
            .build());

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
                let mut str = format!(
                    "{} = {} ({}, type: float, default: {}",
                    cvar.name,
                    cvar.value.as_float(),
                    cvar.description,
                    cvar.default.as_float()
                );
                if let Some(min) = &cvar.min_value {
                    str += &format!(", min: {}", min.as_float());
                }
                if let Some(max) = &cvar.max_value {
                    str += &format!(", max: {}", max.as_float());
                }
                str += ")";
                str
            }
            CvarType::Integer => {
                let mut str = format!(
                    "{} = {} ({}, type: int, default: {}",
                    cvar.name,
                    cvar.value.as_int(),
                    cvar.description,
                    cvar.default.as_int()
                );
                if let Some(min) = &cvar.min_value {
                    str += &format!(", min: {}", min.as_int());
                }
                if let Some(max) = &cvar.max_value {
                    str += &format!(", max: {}", max.as_int());
                }
                str += ")";
                str
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

    pub fn get_trigger(&self, id: u32) -> ControlSignal {
        self.id_to_cvar.get(&id).expect("unknown cvar id").change_trigger
    }

    pub fn get_cvar_id_from_str(&self, cvar_str: &str) -> Option<u32> {
        self.cvar_str_to_id.get(cvar_str).copied()
    }

    pub fn set<T: CvarValue>(&mut self, id: u32, val: T) {
        let cvar = self.id_to_cvar.get_mut(&id).unwrap();

        let mut clamped_value = None;

        if let Some(min_value) = &cvar.min_value {
            if min_value.gt(&val) {
                clamped_value = Some(min_value.as_ref());
            }
        }
        if let Some(max_value) = &cvar.max_value {
            if max_value.lt(&val) {
                clamped_value = Some(max_value.as_ref());
            }
        }

        if let Some(clamped) = clamped_value {
            cvar.value.set(clamped);
        } else {
            cvar.value.set(&val);
        }

        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

struct ConfigVariableBuilder {
    name: Option<&'static str>,
    default: Option<Box<dyn CvarValue>>,
    value: Option<Box<dyn CvarValue>>,
    min_value: Option<Box<dyn CvarValue>>,
    max_value: Option<Box<dyn CvarValue>>,
    description: Option<&'static str>,
    change_trigger: Option<ControlSignal>
}

impl ConfigVariableBuilder {
    fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    fn default<T:CvarValue + Clone + 'static>(mut self, default: T) -> Self {
        self.default = Some(Box::new(default.clone()));
        self.value = Some(Box::new(default));
        self
    }

    fn min_value<T:CvarValue + Clone + 'static>(mut self, min_value: T) -> Self {
        self.min_value = Some(Box::new(min_value));
        self
    }

    fn max_value<T:CvarValue + Clone + 'static>(mut self, max_value: T) -> Self {
        self.max_value = Some(Box::new(max_value));
        self
    }

    fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    fn change_trigger(mut self, change_trigger: ControlSignal) -> Self {
        self.change_trigger = Some(change_trigger);
        self
    }

    fn build(self) -> ConfigVariable {
        if self.name.is_none() || self.default.is_none() {
            panic!("ConfigVariable must contain name and default value");
        }

        let description = if let Some(desc) = self.description { desc } else { "No description"};
        let change_trigger = if let Some(trigger) = self.change_trigger { trigger} else { ControlSignal::None } ;

        ConfigVariable {
            name: self.name.unwrap(),
            value: self.value.unwrap(),
            default: self.default.unwrap(),
            min_value: self.min_value,
            max_value: self.max_value,
            description,
            change_trigger
        }
    }
}

struct ConfigVariable {
    name: &'static str,
    value: Box<dyn CvarValue>,
    min_value: Option<Box<dyn CvarValue>>,
    max_value: Option<Box<dyn CvarValue>>,
    default: Box<dyn CvarValue>,
    description: &'static str,
    change_trigger: ControlSignal
}

impl ConfigVariable {

    pub fn builder() -> ConfigVariableBuilder {
        ConfigVariableBuilder {
            name: None,
            default: None,
            value: None,
            min_value: None,
            max_value: None,
            description: None,
            change_trigger: None
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

    fn gt(&self, rhs: &dyn CvarValue) -> bool;
    fn lt(&self, rhs: &dyn CvarValue) -> bool;
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

    fn gt(&self, rhs: &dyn CvarValue) -> bool {
        *self > rhs.as_float()
    }

    fn lt(&self, rhs: &dyn CvarValue) -> bool {
        *self < rhs.as_float()
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

    fn gt(&self, rhs: &dyn CvarValue) -> bool {
        *self > rhs.as_int()
    }

    fn lt(&self, rhs: &dyn CvarValue) -> bool {
        *self < rhs.as_int()
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

    fn gt(&self, rhs: &dyn CvarValue) -> bool {
        self.len() > rhs.as_str().len()
    }

    fn lt(&self, rhs: &dyn CvarValue) -> bool {
        self.len() < rhs.as_str().len()
    }
}
