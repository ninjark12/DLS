#pragma once

// Number of layers VIA exposes and the dynamic keymap stores in EEPROM.
// Default for most VIA boards is 4; raised to 8 for Dynamic Layer Switcher.
// Each extra layer consumes EEPROM — if VIA fails to load or remaps behave
// oddly after flashing, this is too high for the board's EEPROM and should be
// lowered. The app's via_command_kb handler bounds-checks against this value,
// so the app auto-scales to whatever count is set here.
#define DYNAMIC_KEYMAP_LAYER_COUNT 8
