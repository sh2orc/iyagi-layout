import { describe, expect, it } from "vitest";
import { detectLang, langFromTag, makeT, type Lang } from "./i18n";

describe("language detection", () => {
  it("maps tags to the four supported languages", () => {
    expect(langFromTag("ko-KR")).toBe("ko");
    expect(langFromTag("ja_JP")).toBe("ja");
    expect(langFromTag("zh-Hans-CN")).toBe("zh");
    expect(langFromTag("zh-TW")).toBe("zh");
    expect(langFromTag("en-GB")).toBe("en");
    expect(langFromTag("fr-FR")).toBeNull();
  });
  it("uses the first supported preferred language, else English", () => {
    expect(detectLang(["fr-FR", "ja-JP", "ko-KR"])).toBe("ja");
    expect(detectLang(["de-DE", "es-ES"])).toBe("en");
    expect(detectLang([])).toBe("en");
  });
});

describe("dictionaries", () => {
  it("translate every key into every language", () => {
    const langs: Lang[] = ["en", "ko", "ja", "zh"];
    const en = makeT("en");
    for (const lang of langs) {
      const t = makeT(lang);
      for (const key of ["loading", "save", "quit", "recording", "chainLabel"] as const) {
        expect(t(key).length).toBeGreaterThan(0);
      }
      expect(t("save")).not.toBe("");
      if (lang !== "en") expect(t("save")).not.toBe(en("save"));
    }
  });
});
