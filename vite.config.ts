import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    // iyagi-term이 1420을 쓰므로 두 앱을 동시에 개발할 수 있게 1421.
    port: 1421,
    strictPort: true,
    watch: { ignored: ["**/target/**"] },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: { target: "es2022", minify: "esbuild", sourcemap: false },
});
