# Intro video

30-second motion graphic (English, 1920×1080, 30 fps). `intro.html` draws every frame as a pure
function of time (`window.__render(t)`); `render.mjs` captures it with headless Chrome and encodes
with ffmpeg. The motion follows the real app behavior: same-axis arrows give screen halves, a
perpendicular arrow within 3 s splits the current window again.

```sh
npm i --no-save ffmpeg-static playwright-core     # not added to package.json
node docs/media/intro/render.mjs                  # → docs/media/iyagi-layout-intro.mp4 / .webp
node docs/media/intro/render.mjs --still 8.5,15   # PNG stills for review
```

Storyboard: 0–3.6 logo · 3.6–7 problem · 7–12 shortcut and halves · 12–17.4 follow-up split ·
17.4–21.8 menu bar and settings · 21.8–25.7 principles · 25.7–30 outro.
