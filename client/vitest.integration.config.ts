import { defineConfig } from "vitest/config";

import baseConfig from "./vitest.config";

/**
 * Integration test config. Reuses the base config's plugins, define block,
 * and test environment/setup/pool settings, but overrides `include`/`exclude`
 * so that ONLY `*.integration.test.{ts,tsx}` files run.
 *
 * Note: we do not use `mergeConfig` here because it concatenates arrays —
 * merging would yield both the unit and integration globs in `include` and
 * keep integration files in `exclude`. The `include`/`exclude` fields below
 * are clean overrides, not concatenations.
 */
const base = baseConfig as ReturnType<typeof defineConfig>;

export default defineConfig({
  plugins: base.plugins,
  define: base.define,
  test: {
    ...base.test,
    include: ["src/**/*.integration.test.{ts,tsx}"],
    exclude: ["node_modules/**", "dist/**"],
  },
});
