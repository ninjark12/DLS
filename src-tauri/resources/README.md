# Bundled flash resources

These files are shipped inside the app bundle (declared in
`tauri.conf.json` → `bundle.resources`) and used by `src/firmware.rs` to flash
the Iris LM-K without any local toolchain.

| File | What it is | Status |
|------|------------|--------|
| `dfu-util.exe` | Windows `dfu-util` binary (flashes the STM32 DFU bootloader) | **placeholder (0 bytes)** |
| `iris_lm_dls.bin` | Precompiled `keebio/iris_lm/k1` firmware with the `dls` keymap | **placeholder (0 bytes)** |

Both start as **zero-byte placeholders** so the Tauri bundler doesn't fail on a
missing resource. At runtime, `firmware.rs::is_present()` treats an empty file
as "not bundled yet" and the UI shows a clean message instead of trying to run
an empty binary.

## Phase B — replace the placeholders

1. **`iris_lm_dls.bin`** — build `keebio/iris_lm/k1` with the `dls` keymap from
   `../../firmware/iris_lm_dls/` (via a GitHub Actions workflow or a one-off QMK
   build) and drop the resulting `.bin` here.
2. **`dfu-util.exe`** — download the official Windows `dfu-util` release and
   place the binary here.

No code changes are needed once real (non-empty) files are in place.
