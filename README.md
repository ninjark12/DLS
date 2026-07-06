# Dynamic Layer Switcher

Automatically switch your QMK/VIA keyboard's active layer based on which
application is focused on your PC — e.g. a drawing-tool layer when Clip Studio
Paint is focused, back to your normal layer for everything else.

- **Frontend:** Angular
- **Backend:** Rust + Tauri v2
- **Platform:** Windows (uses Win32 focus hooks + Raw HID)

## How it works

A background thread installs a Win32 foreground-window hook. When focus changes,
it resolves the focused app's executable name, looks up a matching rule in your
config, and sends a Raw HID packet to the keyboard telling it which layer to
activate. The keyboard runs a small custom firmware handler that receives the
packet and moves to that layer.

```
focus change ─► resolve app.exe ─► match rule ─► HID packet ─► keyboard moves layer
```

Layer switching is fire-and-forget over the QMK Raw HID interface (usage page
`0xFF60`). The app works with **any VIA-enabled keyboard** once the firmware
handler (below) is installed — it is not locked to one board.

---

## Download

Grab the latest prebuilt Windows installer from the
[**Releases**](https://github.com/ninjark12/DLS/releases) page:

- **`.msi`** or **`.exe`** installer — download, run, and launch **Dynamic
  Layer Switcher** from the Start menu.

Requires Windows 10/11 and a QMK/VIA-enabled keyboard. Nothing else to install —
the Microsoft Edge WebView2 runtime ships with Windows, and the flashing tools
are bundled inside the app.

After installing, complete the [one-time firmware setup](#one-time-firmware-setup)
so your keyboard can receive layer commands.

> Prefer to build it yourself? See [Running from source](#running-from-source).

---

## Requirements

- Windows 10/11
- A QMK/VIA-enabled keyboard
- Only if building from source: [Rust](https://rustup.rs/) (via rustup),
  [Bun](https://bun.sh/), and the
  [Tauri v2 prerequisites](https://tauri.app/start/prerequisites/) (WebView2 +
  MSVC build tools)

---

## One-time firmware setup

The app moves layers by sending a custom Raw HID command. Your keyboard needs a
tiny firmware handler to receive it. **This does not erase your VIA key remaps** —
those live in EEPROM and survive flashing.

### The handler

VIA already implements `raw_hid_receive` internally. You must **not** override
it — instead hook `via_command_kb`, which VIA calls for commands it doesn't
recognize. The handler (in [`firmware/iris_lm_dls/keymap.c`](firmware/iris_lm_dls/keymap.c))
is board-agnostic:

```c
#include "via.h"
#include "action_layer.h"

#define id_custom_set_layer 0x77   // must match DLS_CMD_SET_LAYER in src-tauri/src/hid.rs

bool via_command_kb(uint8_t *data, uint8_t length) {
    if (data[0] == id_custom_set_layer) {
        uint8_t layer = data[1];
        if (layer < DYNAMIC_KEYMAP_LAYER_COUNT) layer_move(layer);
        return true;   // handled
    }
    return false;      // not ours — let VIA handle it
}
```

### For the Keebio Iris LM-K (bundled one-click flash)

| | |
|---|---|
| QMK path | `keebio/iris_lm/k1` |
| MCU / bootloader | STM32G431 / `stm32-dfu` (DFU id `0483:df11`) |
| VID / PID | `0xCB10` / `0x1756` |

The app can flash this board directly using a bundled `dfu-util` and a
precompiled firmware image — no QMK toolchain required. Open the **Firmware
Setup** card in the app and follow the split-keyboard stepper (flash the left
half, then the right half, resetting each into bootloader mode first).

> **First-flash driver step (Windows):** before `dfu-util` can see the board,
> put a half in bootloader mode (press its reset button), run
> [Zadig](https://zadig.akeo.ie/), select `STM32 BOOTLOADER` (`0483:df11`), and
> install the **WinUSB** driver. One time only.

### For any other VIA keyboard

1. Add the `via_command_kb` handler above to your board's keymap.
2. Ensure `VIA_ENABLE = yes` in that keymap's `rules.mk`.
3. Build and flash it however you normally would (e.g.
   [QMK Toolbox](https://github.com/qmk/qmk_toolbox)).
4. Select the board in the app and use it normally.

---

## Building the firmware image (maintainers)

The bundled flash path needs two files in `src-tauri/resources/` (shipped as
zero-byte placeholders in the repo):

- **`iris_lm_dls.bin`** — built by the GitHub Actions workflow
  [`.github/workflows/build-firmware.yml`](.github/workflows/build-firmware.yml).
  Trigger it from the **Actions** tab (or push a change under `firmware/`), then
  download the `iris_lm_dls-bin` artifact and place `iris_lm_dls.bin` here. The
  workflow assembles the `dls` keymap from the scaffold, compiles
  `keebio/iris_lm/k1` in QMK's CLI container, and uploads the result.
- **`dfu-util.exe`** — download the Windows build from
  [dfu-util releases](http://dfu-util.sourceforge.net/releases/) and drop
  `win64/dfu-util.exe` here.

At runtime the app treats empty placeholders as "not bundled yet" and shows a
clean message instead of the flash button failing.

---

## Running from source

```sh
bun install          # first time
bun run tauri dev    # dev build with hot reload
bun run tauri build  # production bundle (installer in src-tauri/target/release/bundle/)
```

## Usage

1. **Keyboard** — plug in your keyboard and pick it from the list (auto-detected
   via QMK's Raw HID interface).
2. **App Rules** — add rules mapping an executable (e.g. `clipstudiopaint.exe`)
   to a layer number, and set a default layer for everything else.
3. **Status** — press **Start**. Switch focus between apps and the keyboard's
   layer follows. The status card shows the current app and active layer.

## Configuration

Config is stored at `%APPDATA%\dynamic-layer-switcher\config.cfg` (JSON). The app
reads and writes it for you; edits made in the UI are saved immediately and picked
up by a running switcher without a restart.

## Development notes

- Rust tests: `cd src-tauri && cargo test` (the crate is Windows-only).
- Frontend build check: `bun run build`.
- The firmware command byte (`0x77`) must stay in sync between
  `src-tauri/src/hid.rs` and the firmware handler.
