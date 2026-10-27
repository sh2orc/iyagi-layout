import { useEffect, useRef, useState } from "react";
import { prettyShortcut, shortcutFromKeyboardEvent } from "./shortcut";
import { shortcutValidate } from "./api";
import type { T } from "./i18n";

interface Props {
  t: T;
  value: string;
  onChange: (next: string) => void;
  /** 행 밖에서 정한 경고(예: 다른 동작과 같은 조합). 검증 오류가 없을 때 보여 준다. */
  error?: string | null;
}

/** 클릭 → 키 조합을 누르면 그 조합으로 바뀐다. Esc로 취소. */
export function ShortcutRecorder({ t, value, onChange, error }: Props) {
  const [recording, setRecording] = useState(false);
  const [validateError, setValidateError] = useState<string | null>(null);
  const ref = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!recording) return;
    const stop = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
    };
    const onKey = (e: KeyboardEvent) => {
      stop(e);
      if (e.code === "Escape") {
        setRecording(false);
        return;
      }
      // 키를 누르고 있으면 repeat keydown이 계속 온다 — 첫 누름만 녹음한다.
      if (e.repeat) return;
      const spec = shortcutFromKeyboardEvent(e);
      if (!spec) return;
      void shortcutValidate(spec)
        .then(() => {
          setValidateError(null);
          onChange(spec);
        })
        .catch((reason) => setValidateError(String(reason)))
        .finally(() => setRecording(false));
    };
    // 스페이스는 keyup에서 버튼 클릭을 만든다 — Ctrl+Space를 녹음하면 그 클릭이
    // 녹음을 다시 켜므로 keyup도 삼킨다.
    window.addEventListener("keydown", onKey, true);
    window.addEventListener("keyup", stop, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
      window.removeEventListener("keyup", stop, true);
    };
  }, [recording, onChange]);

  const shown = validateError ?? error ?? null;

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
      {shown && <span className="field-error">{shown}</span>}
    </span>
  );
}
