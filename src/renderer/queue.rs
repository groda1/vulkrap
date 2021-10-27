use crate::renderer::surface::SurfaceContainer;
use ash::vk::{PhysicalDevice, QueueFlags};
use std::fmt;
use std::fmt::Display;

pub struct QueueRef {
    pub family_index : u32,
    pub queue_index: u32,
}

pub struct QueueFamilyIndices {
    pub(crate) graphics: QueueRef,
    pub(crate) transfer: QueueRef,
    pub(crate) present: QueueRef,
}

impl QueueFamilyIndices {
    pub fn new(
        instance: &ash::Instance,
        device: &PhysicalDevice,
        surface_container: &SurfaceContainer,
    ) -> QueueFamilyIndices {
        let graphics = pick_graphics_queue_family(instance, device);
        let transfer =  pick_transfer_queue_family(instance, device, graphics.as_ref().unwrap());
        let present = pick_present_queue_family(instance, device, surface_container);

        // TODO: better handling
        assert!(graphics.is_some());
        assert!(transfer.is_some());
        assert!(present.is_some());

        QueueFamilyIndices { graphics: graphics.unwrap(), transfer: transfer.unwrap(), present: present.unwrap() }
    }

}

impl Display for QueueFamilyIndices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(gfx={} {}, present={} {}, transfer={} {})",
            self.graphics.family_index, self.graphics.queue_index,
            self.present.family_index, self.present.queue_index,
            self.transfer.family_index, self.transfer.queue_index,
        )
    }
}

fn pick_graphics_queue_family(instance: &ash::Instance, device: &PhysicalDevice) -> Option<QueueRef> {
    let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(*device) };

    for (index, family_properties) in queue_family_properties.iter().enumerate() {

        if family_properties.queue_flags.contains(QueueFlags::GRAPHICS) {
            return Option::Some(QueueRef { family_index: index as u32, queue_index : 0 });
        }
    }

    Option::None
}

fn pick_transfer_queue_family(instance: &ash::Instance, device: &PhysicalDevice, graphics_queue: &QueueRef) -> Option<QueueRef> {
    let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(*device) };

    for (index, family_properties) in queue_family_properties.iter().enumerate() {
        if family_properties.queue_flags.contains(QueueFlags::TRANSFER) {
            if family_properties.queue_count == 1 && graphics_queue.family_index == index as u32 {
                continue;
            } else if graphics_queue.family_index == index as u32 {
                return  Option::Some(QueueRef { family_index: index as u32, queue_index : 1});
            } else {
                return  Option::Some(QueueRef { family_index: index as u32, queue_index : 0});
            }
        }
    }

    Option::None
}


fn pick_present_queue_family(
    instance: &ash::Instance,
    physical_device: &PhysicalDevice,
    surface_container: &SurfaceContainer,
) -> Option<QueueRef> {
    let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

    for (index, _family_properties) in queue_family_properties.iter().enumerate() {
        let present_support = unsafe {
            surface_container.loader.get_physical_device_surface_support(
                *physical_device,
                index as u32,
                surface_container.surface,
            )
        };

        if present_support.unwrap_or(false) {
            return  Option::Some(QueueRef { family_index: index as u32, queue_index : 0});
        }
    }

    Option::None
}
