use crate::renderer::surface::SurfaceContainer;
use ash::version::InstanceV1_0;
use ash::vk::{PhysicalDevice, QueueFlags};
use std::fmt;
use std::fmt::Display;

pub struct QueueFamilyIndices {
    pub(crate) graphics: Option<u32>,
    pub(crate) present: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new(
        instance: &ash::Instance,
        device: &PhysicalDevice,
        surface_container: &SurfaceContainer,
    ) -> QueueFamilyIndices {
        let graphics = _pick_queue_families(instance, device);
        let present = _pick_present_queue_family(instance, device, surface_container);

        QueueFamilyIndices { graphics, present }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }
}

impl Display for QueueFamilyIndices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(gfx={}, present={})",
            self.graphics.map(|g| g as i32).unwrap_or(-1),
            self.present.map(|g| g as i32).unwrap_or(-1)
        )
    }
}

fn _pick_queue_families(instance: &ash::Instance, device: &PhysicalDevice) -> Option<u32> {
    let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(*device) };

    let mut index = 0;
    for family_properties in queue_family_properties.iter() {
        if family_properties.queue_flags.contains(QueueFlags::GRAPHICS) {
            return Option::Some(index);
        }
        index += 1;
    }

    Option::None
}

fn _pick_present_queue_family(
    instance: &ash::Instance,
    physical_device: &PhysicalDevice,
    surface_container: &SurfaceContainer,
) -> Option<u32> {
    let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

    let mut index = 0;
    for _family_properties in queue_family_properties.iter() {
        let present_support = unsafe {
            surface_container.loader.get_physical_device_surface_support(
                *physical_device,
                index as u32,
                surface_container.surface,
            )
        };

        if present_support.unwrap_or(false) {
            return Option::Some(index);
        }
        index += 1;
    }

    Option::None
}
