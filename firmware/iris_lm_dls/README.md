# `dls` keymap — firmware source for Dynamic Layer Switcher

This is the firmware side of the app: a custom `via_command_kb` handler that lets
the desktop app move the active layer over Raw HID, without breaking the VIA app.

## Board facts (Keebio Iris LM-K)

| | |
|---|---|
| QMK keyboard path | `keebio/iris_lm/k1` |
| MCU | STM32G431 |
| Bootloader | `stm32-dfu` (flashes a `.bin`; DFU id `0483:df11`) |
| PID / VID | `0x1756` / `0xCB10` |
| Command byte | `0x77` (must match `DLS_CMD_SET_LAYER` in `src-tauri/src/hid.rs`) |

## How to build the `dls` keymap

1. In your `qmk_firmware` checkout, copy the stock keymap:
   ```
   cp -r keyboards/keebio/iris_lm/keymaps/default \
         keyboards/keebio/iris_lm/keymaps/dls
   ```
2. **Append** the `via_command_kb` handler from `keymap.c` in this folder to the
   end of `keyboards/keebio/iris_lm/keymaps/dls/keymap.c`. (It is an
   append-only snippet — it relies on the layout array already in the copied
   default keymap.)
3. Copy `rules.mk` from this folder into the same `dls/` directory (adds
   `VIA_ENABLE = yes`).
4. Append `config.h` from this folder to `dls/config.h` (raises
   `DYNAMIC_KEYMAP_LAYER_COUNT` to 8 so VIA exposes 8 layers). `cat >>` creates
   the file if the stock keymap doesn't already have one.
5. Build:
   ```
   qmk compile -kb keebio/iris_lm/k1 -km dls
   ```
   The resulting `.bin` is what the app bundles as
   `src-tauri/resources/iris_lm_dls.bin` (see that folder's README).

## Flashing

The app flashes it for you (bundled `dfu-util`). To flash manually:
```
qmk flash -kb keebio/iris_lm/k1 -km dls
```
Enter the bootloader by pressing the reset button on each half (split board —
flash one half at a time). On Windows the STM32 DFU device may need a one-time
WinUSB driver via Zadig before `dfu-util` can see it.

Your VIA remaps live in EEPROM and **survive the flash** (the compiled-in keymap
is only a fallback; the layer count is unchanged).

## Using this on a different keyboard (generic fallback)

The `via_command_kb` handler in `keymap.c` is **board-agnostic**. To use the app
with any other VIA keyboard, append the same handler block to that board's
keymap, ensure `VIA_ENABLE = yes`, build/flash it however you normally would
(e.g. QMK Toolbox), then select the board in the app. No app changes needed —
device selection and layer sending are already generic across VID/PID.
