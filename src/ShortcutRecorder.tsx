import { useEffect, useRef, useState } from "react";
import { prettyShortcut, shortcutFromKeyboardEvent } from "./shortcut";
import { shortcutValidate } from "./api";
import type { T } from "./i18n";

interface Props {
  t: T;
  value: string;
  onChange: (next: string) => void;
}

/** 클릭 → 키 조합을 누르면 그 조합으로 바뀐다. Esc로 취소. */
export function ShortcutRecorder({ t, value, onChange }: Props) {
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const ref = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!recording) return;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.code === "Escape") {
        setRecording(false);
        return;
      }
      const spec = shortcutFromKeyboardEvent(e);
      if (!spec) return;
      void shortcutValidate(spec)
        .then(() => {
          setError(null);
          onChange(spec);
        })
        .catch((reason) => setError(String(reason)))
        .finally(() => setRecording(false));
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [recording, onChange]);

  return (
    <span className="recorder">
      <button
        ref={ref}
        type="button"
        className={recording ? "recording" : ""}
        onClick={() => setRecording((r) => !r)}
        onBlur={() => setRecording(false)}
        title={value}
      >
        {recording ? t("recording") : prettyShortcut(value)}
      </button>
      {error && <span className="field-error">{error}</span>}
    </span>
  );
}
