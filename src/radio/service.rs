use trouble_host::prelude::*;

// be8e1bad-1ebb-460e-86e2-2c4cdcff5ce6
// const NOTIFIER_CHARACTERISTIC_UUID: BluetoothUuid128 = BluetoothUuid128::new(0xAA);
// 16f4d4d6-cd62-4ab0-a578-43573d92618c
// const STEERING_CHARACTERISTIC_UUID: BluetoothUuid128 = BluetoothUuid128::new(0xBB);

#[gatt_service(uuid = service::INDOOR_POSITIONING)]
pub struct ControlService {
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, name = "device_name", value = "Device name")]
    #[characteristic(uuid = characteristic::DEVICE_NAME, value = [69, 83, 80, 51, 50])]
    // "ESP32"
    pub device_name: [u8; 5],
    #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
    #[characteristic(uuid = "be8e1bad-1ebb-460e-86e2-2c4cdcff5ce6", write, read, notify)]
    pub notifier: bool,

    #[characteristic(uuid = "16f4d4d6-cd62-4ab0-a578-43573d92618c", write)]
    pub steering: u32,
}
