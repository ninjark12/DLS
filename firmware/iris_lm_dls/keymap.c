// Dynamic Layer Switcher — QMK/VIA firmware handler (APPEND-ONLY SNIPPET).
//
// This is NOT a standalone keymap.c. The layer-switch handler below must be
// APPENDED to a copy of your board's existing `default` keymap.c (which already
// contains the `keymaps[][][]` layout array and includes). See README.md for
// exact placement.
//
// It adds a single custom Raw HID command that lets the desktop app move the
// active layer. It hooks VIA's official extension point `via_command_kb`, which
// VIA calls at the top of its own raw_hid_receive: return `true` to claim a
// command, `false` to let VIA process it normally. Do NOT define
// `raw_hid_receive` directly — VIA owns that symbol and redefining it breaks the
// VIA app.
//
// This handler is board-agnostic: the same block works on any VIA-enabled
// keyboard (the generic fallback path). For the Iris LM-K, append it to
//   qmk_firmware/keyboards/keebio/iris_lm/keymaps/dls/keymap.c

#include "via.h"
#include "action_layer.h"

// Must match DLS_CMD_SET_LAYER in the app's src-tauri/src/hid.rs.
#define id_custom_set_layer 0x77

static void via_custom_set_layer(uint8_t *data) {
    uint8_t layer = data[0];
    if (layer < DYNAMIC_KEYMAP_LAYER_COUNT) {
        layer_move(layer);
    }
}

bool via_command_kb(uint8_t *data, uint8_t length) {
    uint8_t *command_id   = &data[0];
    uint8_t *command_data = &data[1];

    switch (*command_id) {
        case id_custom_set_layer:
            via_custom_set_layer(command_data);
            // Note: VIA returns immediately once via_command_kb() reports
            // `true` and does not send a reply, so no ack is emitted here. The
            // app treats layer switching as fire-and-forget.
            return true;
        default:
            return false; // not ours — let VIA handle it normally
    }
}
