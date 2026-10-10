import { describe, expect, it } from "vitest";
import { prettyShortcut, shortcutFromKeyboardEvent } from "./shortcut";

describe("shortcutFromKeyboardEvent", () => {
  it("builds modifier+key strings in a stable order", () => {
    expect(shortcutFromKeyboardEvent({ code: "ArrowLeft", ctrlKey: true, metaKey: true, altKey: false, shiftKey: false }))
      .toBe("Control+Super+ArrowLeft");
    expect(shortcutFromKeyboardEvent({ code: "KeyC", ctrlKey: false, metaKey: true, altKey: true, shiftKey: true }))
      .toBe("Alt+Shift+Super+KeyC");
  });
  it("ignores modifier-only presses and keys without modifiers", () => {
    expect(shortcutFromKeyboardEvent({ code: "MetaLeft", ctrlKey: false, metaKey: true, altKey: false, shiftKey: false })).toBeNull();
    expect(shortcutFromKeyboardEvent({ code: "ArrowLeft", ctrlKey: false, metaKey: false, altKey: false, shiftKey: false })).toBeNull();
  });
});

describe("prettyShortcut", () => {
  it("renders macOS glyphs", () => {
    expect(prettyShortcut("Control+Super+ArrowLeft")).toBe("⌃⌘←");
    expect(prettyShortcut("ctrl+shift+KeyC")).toBe("⌃⇧C");
  });
});
