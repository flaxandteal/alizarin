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
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          "argsIgnorePattern": "^_",
          "varsIgnorePattern": "^_"
        }
      ],
      "prefer-const": "warn",
      "no-empty": "warn",
      "@typescript-eslint/no-unsafe-function-type": "warn",
      "@typescript-eslint/naming-convention": [
        "error",
        {
          "selector": "property",
          "format": ["camelCase", "snake_case"],
          "filter": "^(?!__)",
          "leadingUnderscore": "allow"
        },
        {
          "selector": "property",
          "format": ["camelCase"],
          "filter": "__",
          "prefix": ["__"]
        }
      ]
    },
    extends: [eslint.configs.recommended, tseslint.configs.recommended],
  },
]);
