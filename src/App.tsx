import { useCallback, useEffect, useRef, useState } from "react";
import {
  appInfo, fitSettingsHeight, hideSettings, loginItemSet, onSettingsChanged, openAccessibilitySettings, openLogDir, openLoginItemsSettings,
  permissionPrompt, settingsGet, settingsSet, statusGet,
  type ActionKey, type AppInfo, type Settings, type Status,
} from "./api";
import { ShortcutRecorder } from "./ShortcutRecorder";
import { makeT, systemLang, type Key, type Lang } from "./i18n";

const ACTIONS: Array<{ key: ActionKey; label: Key }> = [
  { key: "left", label: "actionLeft" },
  { key: "right", label: "actionRight" },
  { key: "up", label: "actionUp" },
  { key: "down", label: "actionDown" },
];

const LOGIN_LABEL: Record<Status["loginItem"], Key> = {
  enabled: "loginEnabled",
  requires_approval: "loginRequiresApproval",
  not_registered: "loginNotRegistered",
  not_found: "loginNotFound",
};

export function App() {
  const [draft, setDraft] = useState<Settings | null>(null);
  const [saved, setSaved] = useState<Settings | null>(null);
  const [path, setPath] = useState<string | null>(null);
  const [status, setStatus] = useState<Status | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [info, setInfo] = useState<AppInfo | null>(null);
  // 처음에는 웹뷰 언어로 그리고, 앱이 알려 주는 시스템 언어(메뉴 막대와 같은 값)로 맞춘다.
  const [lang, setLang] = useState<Lang>(systemLang());
  const t = makeT(lang);

  const refreshStatus = useCallback(() => statusGet().then(setStatus), []);
  const pageRef = useRef<HTMLElement>(null);
  const loaded = draft !== null;

  // 창 높이를 내용에 맞춘다. 하단 버튼이 잘리지 않게 하고, 언어나 메시지가 바뀌어 높이가 변해도 따라간다.
  useEffect(() => {
    const page = pageRef.current;
    if (!page) return;
    // 마지막 자식의 아래쪽 + 아래 여백(.page padding 24px)을 기준으로 한다.
    // scrollHeight는 창 높이 아래로 줄지 않으므로 쓰지 않는다.
    const fit = () => {
      const last = page.lastElementChild;
      const contentBottom = last ? last.getBoundingClientRect().bottom : 0;
      // 여백을 63px(기존 48 + 15) 둬서 하단 버튼이 잘리지 않게 한다.
      const height = Math.max(page.offsetHeight, Math.ceil(contentBottom) + 63);
      void fitSettingsHeight(height).catch(() => undefined);
    };
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(page);
    return () => observer.disconnect();
  }, [loaded]);

  useEffect(() => {
    void settingsGet().then((v) => {
      setDraft(v.settings);
      setSaved(v.settings);
      setPath(v.path);
    });
    void appInfo().then((i) => {
      setInfo(i);
      setLang(i.lang);
    });
    void refreshStatus();
    // 권한은 시스템 설정에서 바뀌므로 창이 보이는 동안 주기적으로 다시 읽는다.
    const timer = window.setInterval(() => void refreshStatus(), 2000);
    const unlisten = onSettingsChanged((s) => {
      setSaved(s);
      setDraft(s);
      void refreshStatus();
    });
    return () => {
      window.clearInterval(timer);
      void unlisten.then((f) => f());
    };
  }, [refreshStatus]);

  if (!draft) return <main className="page" ref={pageRef}>{t("loading")}</main>;

  const dirty = JSON.stringify(draft) !== JSON.stringify(saved);

  const save = async () => {
    setSaveError(null);
    try {
      const v = await settingsSet(draft);
      setSaved(v.settings);
      setDraft(v.settings);
    } catch (e) {
      setSaveError(String(e));
    }
  };

  const toggleLogin = async () => {
    if (!status) return;
    try {
      setStatus(await loginItemSet(status.loginItem !== "enabled"));
    } catch (e) {
      setSaveError(String(e));
    }
  };

  const trusted = status?.accessibilityTrusted ?? false;

  return (
    <main className="page" ref={pageRef}>
      <header>
        <h1>IYAGI Layout</h1>
        <p className="muted">{t("subtitle")} {info ? `v${info.version}` : ""}</p>
      </header>

      <section className={trusted ? "" : "attention"}>
        <h2>{t("permTitle")}</h2>
        {trusted ? (
          <p>{t("permOk")}</p>
        ) : (
          <>
            <p>
              {t("permBefore")}
              <b>{t("permPath")}</b>
              {t("permAfter")}
            </p>
            <div className="actions">
              <button className="primary" onClick={() => void permissionPrompt().then(setStatus)}>{t("permRequest")}</button>
              <button onClick={() => void openAccessibilitySettings()}>{t("permOpen")}</button>
            </div>
          </>
        )}
      </section>

      {status?.settingsLoadError && (
        <div className="banner error">{t("loadError")} {status.settingsLoadError}</div>
      )}
      {status && status.hotkeyFailures.length > 0 && (
        <div className="banner error">
          {t("hotkeyFailed")} {status.hotkeyFailures.map((f) => `${f.shortcut} (${f.reason})`).join(", ")}
        </div>
      )}

      <section>
        <h2>{t("shortcutsTitle")}</h2>
        {ACTIONS.map(({ key, label }) => (
          <div key={key} className="row">
            <span>{t(label)}</span>
            <ShortcutRecorder
              t={t}
              value={draft.shortcuts[key]}
              onChange={(next) => setDraft({ ...draft, shortcuts: { ...draft.shortcuts, [key]: next } })}
            />
          </div>
        ))}
        <p className="muted">{t("shortcutsHelp")}</p>
      </section>

      <section>
        <h2>{t("layoutTitle")}</h2>
        <label className="row">
          <span>{t("gapLabel")}</span>
          <input
            type="number"
            min={0}
            max={32}
            value={draft.gap}
            onChange={(e) => setDraft({ ...draft, gap: Math.max(0, Math.min(32, Number(e.target.value) || 0)) })}
          />
        </label>
        <label className="row">
          <span>{t("chainLabel")}</span>
          <input
            type="number"
            min={0}
            max={10}
            step={0.5}
            value={draft.chain_window_ms / 1000}
            onChange={(e) =>
              setDraft({ ...draft, chain_window_ms: Math.round(Math.max(0, Math.min(10, Number(e.target.value) || 0)) * 1000) })
            }
          />
        </label>
        <label className="row">
          <span>{t("cycleLabel")}</span>
          <input type="checkbox" checked={draft.cycle_thirds} onChange={(e) => setDraft({ ...draft, cycle_thirds: e.target.checked })} />
        </label>
      </section>

      <section>
        <h2>{t("loginTitle")}</h2>
        <div className="row">
          <span>{status ? t(LOGIN_LABEL[status.loginItem]) : "…"}</span>
          <span className="actions">
            {status?.loginItem === "requires_approval" && (
              <button onClick={() => void openLoginItemsSettings()}>{t("loginOpenSettings")}</button>
            )}
            <button disabled={!status} onClick={() => void toggleLogin()}>
              {status?.loginItem === "enabled" ? t("loginOff") : t("loginOn")}
            </button>
          </span>
        </div>
      </section>

      <footer className="actions">
        <button onClick={() => void openLogDir()}>{t("openLogs")}</button>
        <button onClick={() => void hideSettings()} title={t("hideHint")}>{t("hideToMenuBar")}</button>
        <span className="muted path" title={path ?? ""}>{path ?? ""}</span>
        <button className="primary" disabled={!dirty} onClick={() => void save()}>{t("save")}</button>
      </footer>
      {saveError && <div className="banner error">{saveError}</div>}
    </main>
  );
}
