// find keyboards
// list keyboards to choose the correct keyboard
// get keyboard info
//  includes hid, vid, product desc
// send the keyboard info to switcher
//
use hidapi::HidApi;
use serde::{Deserialize, Serialize};

/// Raw HID command byte. Must match `DLS_CMD_SET_LAYER` in the QMK
/// `raw_hid_receive_kb` handler. Chosen outside Via's reserved range
/// (0x01-0x35) so Via forwards it to the keyboard-level fallback.
const DLS_CMD_SET_LAYER: u8 = 0xE0;
const PACKET_SIZE: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardInfo {
    pub pid: u16,
    pub vid: u16,
    pub product_string: String,
}

/// Build the 32-byte Raw HID packet the firmware expects:
///   byte 0 = command, byte 1 = target layer, rest zeroed.
fn build_layer_packet(layer_number: u8) -> [u8; PACKET_SIZE] {
    let mut packet = [0u8; PACKET_SIZE];
    packet[0] = DLS_CMD_SET_LAYER;
    packet[1] = layer_number;
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
    fn packet_has_correct_command_byte() {
        let packet = build_layer_packet(3);
        assert_eq!(packet[0], DLS_CMD_SET_LAYER);
        assert_eq!(packet[0], 0xE0);
    }

    #[test]
    fn packet_carries_target_layer() {
        assert_eq!(build_layer_packet(0)[1], 0);
        assert_eq!(build_layer_packet(7)[1], 7);
        assert_eq!(build_layer_packet(255)[1], 255);
    }

    #[test]
    fn packet_is_32_bytes_and_padded_with_zeros() {
        let packet = build_layer_packet(5);
        assert_eq!(packet.len(), 32);
        assert!(packet[2..].iter().all(|&b| b == 0));
    }

    #[test]
    fn command_byte_is_outside_via_reserved_range() {
        // Via reserves 0x01-0x35 for its own protocol commands.
        assert!(DLS_CMD_SET_LAYER > 0x35);
    }
}
