// @ts-check

import eslint from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores(["dist/*"]),
  {
    languageOptions: {
      globals: {
        fetch: false,
        assert: false,
      },
    },
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-unused-vars": "warn",
      "prefer-const": "warn",
      "no-empty": "warn",
      "@typescript-eslint/no-unsafe-function-type": "warn",
      "@typescript-eslint/naming-convention": [
        "error",
        {
          "leadingUnderscore": "allow"
        }
      ]
    },
    extends: [eslint.configs.recommended, tseslint.configs.recommended],
  },
]);
