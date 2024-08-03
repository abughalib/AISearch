use candle_core::Device;

pub fn get_device() -> Device {
    if let Ok(cuda) = Device::cuda_if_available(0) {
        return cuda;
    }

    return Device::Cpu;
}
