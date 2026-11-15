import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The dev server is only ever reached by the desktop shell's webview, never over a network.
// 5174, not Vite's 5173, so it can run beside Learning Corner's dev server.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5174, strictPort: true },
  build: { target: "es2022", sourcemap: true },
});
