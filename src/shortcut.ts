/** 키보드 이벤트 → global-hotkey 문자열(`Control+Super+ArrowLeft`). 수정키만 눌렸으면 null. */
export function shortcutFromKeyboardEvent(e: {
  code: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}): string | null {
  const mods: string[] = [];
  if (e.ctrlKey) mods.push("Control");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  if (e.metaKey) mods.push("Super");
  const key = e.code;
  if (!key || MODIFIER_CODES.has(key) || mods.length === 0) return null;
  return [...mods, key].join("+");
}

const MODIFIER_CODES = new Set([
  "ControlLeft", "ControlRight", "ShiftLeft", "ShiftRight", "AltLeft", "AltRight",
  "MetaLeft", "MetaRight", "CapsLock", "Fn", "FnLock", "OSLeft", "OSRight",
]);

/** `Control+Super+ArrowLeft` → `⌃⌘←` (트레이와 같은 표기). */
export function prettyShortcut(spec: string): string {
  return spec
    .split("+")
    .map((t) => {
      const u = t.trim().toUpperCase();
      switch (u) {
        case "CONTROL": case "CTRL": return "⌃";
        case "SUPER": case "COMMAND": case "CMD": case "COMMANDORCONTROL": case "CMDORCTRL": return "⌘";
        case "ALT": case "OPTION": return "⌥";
        case "SHIFT": return "⇧";
        case "ARROWLEFT": case "LEFT": return "←";
        case "ARROWRIGHT": case "RIGHT": return "→";
        case "ARROWUP": case "UP": return "↑";
        case "ARROWDOWN": case "DOWN": return "↓";
        case "ENTER": case "RETURN": return "↩";
        case "BACKSPACE": return "⌫";
        case "SPACE": return "␣";
        default: return u.startsWith("KEY") ? u.slice(3) : u.startsWith("DIGIT") ? u.slice(5) : t.trim();
      }
    })
    .join("");
}
