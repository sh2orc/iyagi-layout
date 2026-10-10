// 설정 창 문구. Rust 쪽(메뉴 막대) 문구는 crates/layout-core/src/i18n.rs.
// 언어는 시스템 선호 언어에서 정한다: 지원하는 첫 언어(en/ko/ja/zh), 없으면 영어.
// 중국어는 간체로 통일한다.

export type Lang = "en" | "ko" | "ja" | "zh";

export function langFromTag(tag: string): Lang | null {
  const primary = tag.trim().split(/[-_]/)[0]?.toLowerCase();
  return primary === "en" || primary === "ko" || primary === "ja" || primary === "zh" ? primary : null;
}

export function detectLang(tags: readonly string[]): Lang {
  for (const tag of tags) {
    const lang = langFromTag(tag);
    if (lang) return lang;
  }
  return "en";
}

const en = {
  loading: "Loading…",
  subtitle: "Snap the current window to a half of the screen with a shortcut.",
  permTitle: "Accessibility permission",
  permOk: "Allowed — IYAGI Layout can move other apps' windows.",
  permBefore: "To move other apps' windows, allow IYAGI Layout under ",
  permPath: "System Settings › Privacy & Security › Accessibility",
  permAfter: ". This page updates automatically once it is allowed.",
  permRequest: "Request permission",
  permOpen: "Open Settings",
  loadError: "The settings file could not be read, so defaults are in use:",
  hotkeyFailed: "Shortcuts that could not be registered:",
  shortcutsTitle: "Shortcuts",
  shortcutsHelp: "Press a button, then press the combination you want. Change it here if it clashes with another app's shortcut.",
  actionLeft: "Left half",
  actionRight: "Right half",
  actionUp: "Top half",
  actionDown: "Bottom half",
  layoutTitle: "Layout",
  gapLabel: "Gap between screen edge and windows (pt)",
  chainLabel: "Follow-up window (seconds) — press a perpendicular arrow within this time to split the current window again, e.g. left then down = bottom-left quarter. 0 turns it off",
  cycleLabel: "Repeat the same key to cycle 1/2 → 2/3 → 1/3",
  loginTitle: "Launch at login",
  loginEnabled: "On",
  loginRequiresApproval: "Turned off in System Settings — allow it under Login Items",
  loginNotRegistered: "Off",
  loginNotFound: "Off (status unavailable — press Turn on to try registering)",
  loginOpenSettings: "Open Login Items",
  loginOff: "Turn off",
  loginOn: "Turn on",
  openLogs: "Open logs folder",
  quit: "Quit app",
  quitHint: "Quits the app completely. Shortcuts stop working.",
  save: "Save",
  recording: "Press keys… (Esc to cancel)",
} as const;

export type Key = keyof typeof en;
type Dict = Record<Key, string>;

const ko: Dict = {
  loading: "불러오는 중…",
  subtitle: "단축키로 현재 창을 화면 절반에 맞춥니다.",
  permTitle: "손쉬운 사용 권한",
  permOk: "허용됨 — 다른 앱의 창을 옮길 수 있습니다.",
  permBefore: "다른 앱의 창을 옮기려면 ",
  permPath: "시스템 설정 › 개인정보 보호 및 보안 › 손쉬운 사용",
  permAfter: "에서 IYAGI Layout을 허용해야 합니다. 허용하면 이 화면이 자동으로 갱신됩니다.",
  permRequest: "권한 요청",
  permOpen: "설정 열기",
  loadError: "설정 파일을 읽지 못해 기본값으로 동작 중입니다:",
  hotkeyFailed: "등록하지 못한 단축키:",
  shortcutsTitle: "단축키",
  shortcutsHelp: "버튼을 누른 뒤 원하는 조합을 누르세요. 다른 앱의 단축키와 겹치면 여기서 바꿀 수 있습니다.",
  actionLeft: "왼쪽 절반",
  actionRight: "오른쪽 절반",
  actionUp: "위 절반",
  actionDown: "아래 절반",
  layoutTitle: "배치",
  gapLabel: "화면 가장자리·창 사이 간격 (pt)",
  chainLabel: "연속 입력 인식 시간 (초) — 이 시간 안에 방향이 다른 키를 누르면 현재 창을 다시 나눕니다. 왼쪽 다음 아래 = 왼쪽 아래 1/4. 0이면 끔",
  cycleLabel: "같은 키를 반복하면 1/2 → 2/3 → 1/3 순환",
  loginTitle: "로그인 시 실행",
  loginEnabled: "켜짐",
  loginRequiresApproval: "시스템 설정에서 꺼져 있음 — 로그인 항목에서 허용해야 합니다",
  loginNotRegistered: "꺼짐",
  loginNotFound: "꺼짐 (상태를 읽지 못함 — 켜기를 눌러 등록을 시도합니다)",
  loginOpenSettings: "로그인 항목 열기",
  loginOff: "끄기",
  loginOn: "켜기",
  openLogs: "로그 폴더 열기",
  quit: "앱 종료",
  quitHint: "앱을 완전히 종료합니다. 단축키도 멈춥니다.",
  save: "저장",
  recording: "키를 누르세요… (Esc 취소)",
};

const ja: Dict = {
  loading: "読み込み中…",
  subtitle: "ショートカットで現在のウインドウを画面の半分に合わせます。",
  permTitle: "アクセシビリティの許可",
  permOk: "許可済み — 他のアプリのウインドウを移動できます。",
  permBefore: "他のアプリのウインドウを移動するには、",
  permPath: "システム設定 › プライバシーとセキュリティ › アクセシビリティ",
  permAfter: " で IYAGI Layout を許可してください。許可すると、この画面は自動的に更新されます。",
  permRequest: "許可をリクエスト",
  permOpen: "設定を開く",
  loadError: "設定ファイルを読み込めなかったため、初期値で動作しています:",
  hotkeyFailed: "登録できなかったショートカット:",
  shortcutsTitle: "ショートカット",
  shortcutsHelp: "ボタンを押してから、割り当てたいキーの組み合わせを押してください。他のアプリと重なる場合はここで変更できます。",
  actionLeft: "左半分",
  actionRight: "右半分",
  actionUp: "上半分",
  actionDown: "下半分",
  layoutTitle: "配置",
  gapLabel: "画面の端・ウインドウ間の余白 (pt)",
  chainLabel: "連続入力の受付時間 (秒) — この時間内に別方向の矢印キーを押すと、現在のウインドウをさらに分割します。左→下 = 左下 1/4。0 で無効",
  cycleLabel: "同じキーを繰り返すと 1/2 → 2/3 → 1/3 と切り替え",
  loginTitle: "ログイン時に起動",
  loginEnabled: "オン",
  loginRequiresApproval: "システム設定でオフになっています — ログイン項目で許可してください",
  loginNotRegistered: "オフ",
  loginNotFound: "オフ (状態を取得できません — オンを押して登録を試みます)",
  loginOpenSettings: "ログイン項目を開く",
  loginOff: "オフにする",
  loginOn: "オンにする",
  openLogs: "ログフォルダを開く",
  quit: "アプリを終了",
  quitHint: "アプリを完全に終了します。ショートカットも無効になります。",
  save: "保存",
  recording: "キーを押してください… (Esc でキャンセル)",
};

const zh: Dict = {
  loading: "加载中…",
  subtitle: "使用快捷键将当前窗口贴合到屏幕的一半。",
  permTitle: "辅助功能权限",
  permOk: "已允许 — 可以移动其他应用的窗口。",
  permBefore: "要移动其他应用的窗口，请在",
  permPath: "系统设置 › 隐私与安全性 › 辅助功能",
  permAfter: "中允许 IYAGI Layout。允许后此页面会自动更新。",
  permRequest: "请求权限",
  permOpen: "打开设置",
  loadError: "无法读取设置文件，当前使用默认值：",
  hotkeyFailed: "无法注册的快捷键：",
  shortcutsTitle: "快捷键",
  shortcutsHelp: "点击按钮后按下想要的组合键。如果与其他应用的快捷键冲突，可在此更改。",
  actionLeft: "左半屏",
  actionRight: "右半屏",
  actionUp: "上半屏",
  actionDown: "下半屏",
  layoutTitle: "布局",
  gapLabel: "屏幕边缘与窗口之间的间距 (pt)",
  chainLabel: "连续输入识别时间 (秒) — 在此时间内按下不同方向的方向键，会再次分割当前窗口。先左后下 = 左下四分之一。0 表示关闭",
  cycleLabel: "重复按同一个键，在 1/2 → 2/3 → 1/3 之间循环",
  loginTitle: "登录时启动",
  loginEnabled: "已开启",
  loginRequiresApproval: "已在系统设置中关闭 — 请在登录项中允许",
  loginNotRegistered: "已关闭",
  loginNotFound: "已关闭 (无法读取状态 — 点击开启以尝试注册)",
  loginOpenSettings: "打开登录项",
  loginOff: "关闭",
  loginOn: "开启",
  openLogs: "打开日志文件夹",
  quit: "退出应用",
  quitHint: "完全退出应用，快捷键也会停止工作。",
  save: "保存",
  recording: "请按下按键… (按 Esc 取消)",
};

const dictionaries: Record<Lang, Dict> = { en, ko, ja, zh };

export type T = (key: Key) => string;

export function makeT(lang: Lang): T {
  const dict = dictionaries[lang];
  return (key) => dict[key];
}

export function systemLang(): Lang {
  const tags = typeof navigator === "undefined" ? [] : navigator.languages?.length ? navigator.languages : [navigator.language ?? ""];
  return detectLang(tags);
}
