// find keyboards
// list keyboards to choose the correct keyboard
// get keyboard info
//  includes hid, vid, product desc
// send the keyboard info to switcher
//
use hidapi::{DeviceInfo, HidApi};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardInfo {
    pub pid: u16,
    pub vid: u16,
    pub product_string: String,
}

pub fn list_keyboards() -> Vec<KeyboardInfo> {
    let mut keyboards: Vec<KeyboardInfo> = Vec::new();
    let api = HidApi::new();
    match api {
        Ok(api) => {
            for device in api.device_list() {
                if device.usage_page() != 0xFF60 {
                    continue;
                }
                keyboards.push(KeyboardInfo {
                    pid: device.product_id(),
                    vid: device.vendor_id(),
                    product_string: device
                        .product_string()
                        .unwrap_or("No product description.")
                        .to_string(),
                });
            }
        }
        Err(e) => eprintln!("couldn't get devices: {}", e),
    }
    keyboards
}

pub fn send_layer(
    current_keyboard_pid: u16,
    current_keyboard_vid: u16,
    layer_number: u8,
) -> Result<(), String> {
    let api = HidApi::new();
    match api {
        Ok(api) => {
            let device = api
                .open(current_keyboard_vid, current_keyboard_pid)
                .map_err(|e| e.to_string())?;
            let mut packet = [0u8; 32];
            packet[0] = 0x01;
            packet[1] = layer_number;
            device.write(&packet).map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => return Err(e.to_string()),
    }
}
