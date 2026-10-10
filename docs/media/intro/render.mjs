#!/usr/bin/env node
// render.mjs — intro.html을 프레임별로 캡처해 MP4(+미리보기 WebP)로 인코딩한다.
//
//   npm i --no-save ffmpeg-static playwright-core     # package.json에는 넣지 않는다
//   node docs/media/intro/render.mjs                  # → docs/media/iyagi-layout-intro.mp4/.webp
//   node docs/media/intro/render.mjs --still 8.5,15   # 그 시각의 PNG만 저장(확인용)
//
// 브라우저는 시스템 Chrome을 쓴다(IYAGI_CHROME로 경로 지정 가능).

import { spawn, execFileSync } from "node:child_process";
import { mkdirSync, existsSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../../..");
const require = createRequire(join(root, "package.json"));
const { chromium } = require("playwright-core");
const ffmpeg = require("ffmpeg-static");

const FPS = 30;
const SECONDS = 30;
const out = join(root, "docs/media");
const args = process.argv.slice(2);
const stillArg = args.indexOf("--still");
const chrome = process.env.IYAGI_CHROME ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

const browser = await chromium.launch({ executablePath: chrome, args: ["--force-device-scale-factor=1", "--allow-file-access-from-files"] });
const page = await browser.newPage({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
await page.goto(pathToFileURL(join(here, "intro.html")).href);
await page.waitForFunction(() => typeof window.__render === "function");
await page.evaluate(() => document.fonts.ready);

if (stillArg !== -1) {
  const dir = join(process.env.IYAGI_STILL_DIR ?? out, "stills");
  mkdirSync(dir, { recursive: true });
  for (const t of args[stillArg + 1].split(",").map(Number)) {
    await page.evaluate((x) => window.__render(x), t);
    const file = join(dir, `t${String(t).replace(".", "_")}.png`);
    writeFileSync(file, await page.screenshot({ type: "png" }));
    console.log("wrote", file);
  }
  await browser.close();
  process.exit(0);
}

mkdirSync(out, { recursive: true });
const mp4 = join(out, "iyagi-layout-intro.mp4");
const enc = spawn(ffmpeg, [
  "-y", "-loglevel", "error", "-f", "image2pipe", "-framerate", String(FPS), "-c:v", "png", "-i", "-",
  "-c:v", "libx264", "-preset", "slow", "-crf", "17", "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-r", String(FPS), mp4,
], { stdio: ["pipe", "inherit", "inherit"] });
const done = new Promise((res, rej) => { enc.on("exit", (c) => (c === 0 ? res() : rej(new Error("ffmpeg exited " + c)))); });

const total = FPS * SECONDS;
for (let f = 0; f < total; f++) {
  await page.evaluate((t) => window.__render(t), f / FPS);
  const png = await page.screenshot({ type: "png" });
  if (!enc.stdin.write(png)) await new Promise((r) => enc.stdin.once("drain", r));
  if (f % 90 === 0) console.log(`frame ${f}/${total}`);
}
enc.stdin.end();
await done;
await browser.close();

// 미리보기: README에서 MP4로 연결되는 가벼운 애니메이션 WebP.
const webp = join(out, "iyagi-layout-intro.webp");
execFileSync(ffmpeg, ["-y", "-loglevel", "error", "-i", mp4, "-vf", "fps=12,scale=800:-1:flags=lanczos", "-c:v", "libwebp", "-quality", "70", "-loop", "0", webp]);
console.log("wrote", mp4, webp);
