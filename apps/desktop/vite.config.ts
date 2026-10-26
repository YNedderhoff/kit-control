import { defineConfig } from "vitest/config";

export default defineConfig({
  server: { port: 1420, strictPort: true },
  clearScreen: false,
  test: {
    include: ["src/**/*.test.{ts,tsx}"],
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
  },
});
