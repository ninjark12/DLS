#pragma once

// Enlarge the emulated-EEPROM (wear-leveling) region so 8 dynamic-keymap
// layers fit. The stock default is too small — 8 layers overflow it and the
// build fails with a static assertion ("Dynamic keymaps are configured to use
// more EEPROM than is available."). The STM32G431 has 128KB flash with room to
// spare, so we roughly double the backing store. Backing size must be a
// multiple of the 2KB flash page; logical size must be <= backing / 2.
#define WEAR_LEVELING_BACKING_SIZE 8192
#define WEAR_LEVELING_LOGICAL_SIZE 4096

// Number of layers VIA exposes and the dynamic keymap stores in EEPROM.
// Default for most VIA boards is 4; raised to 8 for Dynamic Layer Switcher.
// The app's via_command_kb handler bounds-checks against this value, so the
// app auto-scales to whatever count is set here.
#define DYNAMIC_KEYMAP_LAYER_COUNT 8
