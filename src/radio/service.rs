use trouble_host::prelude::characteristic::BATTERY_LEVEL;
use trouble_host::prelude::characteristic
use trouble_host::prelude::*;

const NOTIFIER_CHARACTERISTIC_UUID: BluetoothUuid128 =
    BluetoothUuid128::try_from("be8e1bad-1ebb-460e-86e2-2c4cdcff5ce6");
const STEERING_CHARACTERISTIC_UUID: BluetoothUuid128 =
    BluetoothUuid128::from("16f4d4d6-cd62-4ab0-a578-43573d92618c");

#[gatt_service(uuid = INDOOR_POSITIONING)]
pub struct ControlService {
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, name = "device_name", value = "Device name")]
    #[characteristic(uuid = characteristic::DEVICE_NAME, value = "ESP32")]
    pub device_name: u8,

    #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
    #[characteristic(uuid = NOTIFIER_CHARACTERISTIC_UUID, write, read, notify)]
    pub notifier: bool,

    #[characteristic(uuid = STEERING_CHARACTERISTIC_UUID, write)]
    pub steering: u32,
}
