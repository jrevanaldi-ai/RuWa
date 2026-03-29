// Re-export commands from core
pub use wacore_ng::store::commands::*;

// Wrapper function to apply commands to our platform-specific Device
pub fn apply_command_to_device(device: &mut crate::store::Device, command: DeviceCommand) {
    wacore_ng::store::commands::apply_command_to_device(&mut device.core, command);
}
