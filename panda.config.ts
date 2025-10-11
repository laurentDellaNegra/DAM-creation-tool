import { defineConfig } from "@pandacss/dev";
import { createPreset } from "@park-ui/panda-preset";
import orange from "@park-ui/panda-preset/colors/orange";
import sand from "@park-ui/panda-preset/colors/sand";
// import { recipes, slotRecipes } from "~/theme/recipes";

export default defineConfig({
  preflight: true,
  presets: [
    createPreset({
      accentColor: orange,
      grayColor: sand,
      radius: "lg",
    }),
  ],
  include: ["./src/**/*.{js,jsx,ts,tsx,vue}"],
  jsxFramework: "react",
  outdir: "styled-system",
  theme: {
    extend: {
      // recipes,
      // slotRecipes,
    },
  },
});
