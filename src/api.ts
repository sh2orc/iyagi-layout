import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** Rust `layout_core::Settings`와 같은 모양(serde 기본 표기 = snake_case). */
export interface Settings {
  version: number;
  shortcuts: { left: string; right: string; up: string; down: string };
  gap: number;
  cycle_thirds: boolean;
  chain_window_ms: number;
  enhanced_ui_policy: "automatic" | "disable_enable" | "disable_only" | "off";
  log_level: "error" | "warn" | "info" | "debug" | "trace";
}

export type ActionKey = keyof Settings["shortcuts"];

export interface SettingsView {
  settings: Settings;
  path: string | null;
  loadError: string | null;
}

export interface AppInfo {
  version: string;
  logDir: string | null;
  lang: "en" | "ko" | "ja" | "zh";
}

export type LoginItemStatus = "enabled" | "requires_approval" | "not_registered" | "not_found";

export interface HotkeyFailure {
  action: string;
  shortcut: string;
  reason: string;
}

export interface Status {
  accessibilityTrusted: boolean;
  loginItem: LoginItemStatus;
  hotkeyFailures: HotkeyFailure[];
  settingsLoadError: string | null;
}

export const settingsGet = () => invoke<SettingsView>("settings_get");
export const settingsSet = (settings: Settings) => invoke<SettingsView>("settings_set", { settings });
export const appInfo = () => invoke<AppInfo>("app_info");
export const openLogDir = () => invoke<void>("open_log_dir");
export const statusGet = () => invoke<Status>("status_get");
export const permissionPrompt = () => invoke<Status>("permission_prompt");
export const openAccessibilitySettings = () => invoke<void>("open_accessibility_settings");
export const loginItemSet = (enabled: boolean) => invoke<Status>("login_item_set", { enabled });
export const openLoginItemsSettings = () => invoke<void>("open_login_items_settings");
export const hideSettings = () => invoke<void>("hide_settings");
export const fitSettingsHeight = (height: number) => invoke<void>("fit_settings_height", { height });
export const shortcutValidate = (spec: string) => invoke<string>("shortcut_validate", { spec });

export const onSettingsChanged = (handler: (s: Settings) => void): Promise<UnlistenFn> =>
  listen<Settings>("iyagi-layout://settings-changed", (e) => handler(e.payload));
