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

  async saveConfig() {
    try {
      await invoke("save_config", { newConfig: this.config });
      this.error = null;
    } catch (e) {
      this.error = String(e);
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
}
