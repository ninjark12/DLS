import { Component, NgZone, OnInit, OnDestroy } from "@angular/core";
import { FormsModule } from "@angular/forms";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

interface KeyboardInfo {
  pid: number;
  vid: number;
  product_string: string;
}

interface AppRule {
  exe: string;
  layer: number;
  label: string;
}

interface Config {
  vendor_id: number;
  product_id: number;
  default_layer: number;
  rules: AppRule[];
}

interface SwitcherStatus {
  current_exe: string;
  current_layer: number;
}

@Component({
  selector: "app-root",
  standalone: true,
  imports: [FormsModule],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent implements OnInit, OnDestroy {
  devices: KeyboardInfo[] = [];
  config: Config = { vendor_id: 0, product_id: 0, default_layer: 0, rules: [] };
  running = false;
  status: SwitcherStatus | null = null;
  newRule: AppRule = { exe: "", layer: 0, label: "" };
  error: string | null = null;

  // ── Save Rules button state ──
  rulesDirty = false;   // enable the button only after an inline edit
  rulesSaving = false;
  rulesSaved = false;   // show the "✓ Saved" confirmation briefly
  private savedTimer?: ReturnType<typeof setTimeout>;

  // ── Firmware Setup card ──
  firmwareOpen = false;
  showOtherKeyboards = false;
  flashStep: "idle" | "left" | "right" | "done" = "idle";
  flashing = false;
  flashOutput: string | null = null;
  flashError: string | null = null;

  private unlisten?: UnlistenFn;

  constructor(private ngZone: NgZone) {}

  async ngOnInit() {
    await this.loadDevices();
    this.config = await invoke<Config>("get_config");
    this.running = await invoke<boolean>("get_status");
    // listen() fires outside Angular's zone — ngZone.run() triggers change detection
    this.unlisten = await listen<SwitcherStatus>("switcher-status", (e) => {
      this.ngZone.run(() => {
        this.status = e.payload;
      });
    });
  }

  ngOnDestroy() {
    this.unlisten?.();
    if (this.savedTimer) clearTimeout(this.savedTimer);
  }

  async loadDevices() {
    this.devices = await invoke<KeyboardInfo[]>("list_devices");
  }

  isSelected(device: KeyboardInfo): boolean {
    return device.vid === this.config.vendor_id && device.pid === this.config.product_id;
  }

  async selectDevice(device: KeyboardInfo) {
    this.config.vendor_id = device.vid;
    this.config.product_id = device.pid;
    await this.saveConfig();
  }

  async saveConfig(): Promise<boolean> {
    try {
      await invoke("save_config", { newConfig: this.config });
      this.error = null;
      this.rulesDirty = false; // any successful save persists the whole config
      return true;
    } catch (e) {
      this.error = String(e);
      return false;
    }
  }

  /** Called on every inline edit to a rule cell — arms the Save button. */
  markRulesDirty() {
    this.rulesDirty = true;
    this.rulesSaved = false; // a new edit invalidates the old confirmation
    if (this.savedTimer) clearTimeout(this.savedTimer);
  }

  /** Save via the button: persist, confirm, then disarm the button again. */
  async saveRules() {
    this.rulesSaving = true;
    const ok = await this.saveConfig();
    this.rulesSaving = false;
    if (ok) {
      this.rulesSaved = true;
      this.savedTimer = setTimeout(() => (this.rulesSaved = false), 2500);
    }
  }

  async addRule() {
    if (!this.newRule.exe.trim()) return;
    this.config.rules.push({ ...this.newRule });
    await this.saveConfig();
    this.newRule = { exe: "", layer: 0, label: "" };
  }

  async removeRule(index: number) {
    this.config.rules.splice(index, 1);
    await this.saveConfig();
  }

  async startSwitcher() {
    try {
      await invoke("start_switcher");
      this.running = true;
      this.error = null;
    } catch (e) {
      this.error = String(e);
    }
  }

  async stopSwitcher() {
    await invoke("stop_switcher");
    this.running = false;
    this.status = null;
  }

  toHex(n: number): string {
    return "0x" + n.toString(16).toUpperCase().padStart(4, "0");
  }

  // ── Firmware flashing ──
  startFlashFlow() {
    this.flashStep = "left";
    this.flashOutput = null;
    this.flashError = null;
  }

  resetFlashFlow() {
    this.flashStep = "idle";
    this.flashing = false;
    this.flashOutput = null;
    this.flashError = null;
  }

  /** Flash whichever half is currently connected + in bootloader mode. */
  async flashHalf(next: "right" | "done") {
    this.flashing = true;
    this.flashOutput = null;
    this.flashError = null;
    try {
      this.flashOutput = await invoke<string>("flash_half");
      this.flashStep = next;
    } catch (e) {
      this.flashError = String(e);
    } finally {
      this.flashing = false;
    }
  }
}
