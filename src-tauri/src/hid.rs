// find keyboards
// list keyboards to choose the correct keyboard
// get keyboard info
//  includes hid, vid, product desc
// send the keyboard info to switcher
//
use hidapi::HidApi;
use serde::{Deserialize, Serialize};

/// Raw HID command byte. Must match `id_custom_set_layer` in the QMK
/// `via_command_kb` handler. Chosen outside Via's own command range so our
/// `via_command_kb` override can claim it before Via inspects the packet.
const DLS_CMD_SET_LAYER: u8 = 0x77;

/// QMK Raw HID endpoint payload size (RAW_EPSIZE). This is what the firmware
/// actually receives in `data[0..32]`.
const REPORT_SIZE: usize = 32;

/// Host write buffer = report ID byte + payload. On Windows, hidapi treats
/// byte 0 of the write buffer as the HID Report ID and strips it before the
/// data reaches the device. QMK's raw endpoint has no report ID, so byte 0
/// must be 0x00; the firmware then sees our command at data[0].
const WRITE_SIZE: usize = REPORT_SIZE + 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardInfo {
    pub pid: u16,
    pub vid: u16,
    pub product_string: String,
}

/// Build the host write buffer:
///   byte 0 = 0x00 report ID (stripped by the OS HID stack)
///   byte 1 = command (firmware sees this as data[0])
///   byte 2 = target layer (firmware sees this as data[1])
///   rest   = zero padding
fn build_layer_packet(layer_number: u8) -> [u8; WRITE_SIZE] {
    let mut packet = [0u8; WRITE_SIZE];
    packet[0] = 0x00; // HID report ID
    packet[1] = DLS_CMD_SET_LAYER;
    packet[2] = layer_number;
    packet
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
            let packet = build_layer_packet(layer_number);
            device.write(&packet).map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => return Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_leads_with_zero_report_id() {
        // Windows hidapi strips byte 0 as the HID report ID; it must be 0x00.
        assert_eq!(build_layer_packet(3)[0], 0x00);
    }

    #[test]
    fn command_byte_follows_report_id() {
        let packet = build_layer_packet(3);
        assert_eq!(packet[1], DLS_CMD_SET_LAYER);
        assert_eq!(packet[1], 0x77);
    }

    #[test]
    fn target_layer_follows_command() {
        assert_eq!(build_layer_packet(0)[2], 0);
        assert_eq!(build_layer_packet(7)[2], 7);
        assert_eq!(build_layer_packet(255)[2], 255);
    }

    #[test]
    fn write_buffer_is_report_id_plus_full_payload() {
        let packet = build_layer_packet(5);
        assert_eq!(packet.len(), 33); // 1 report ID + 32-byte RAW_EPSIZE payload
        assert!(packet[3..].iter().all(|&b| b == 0));
    }
}
