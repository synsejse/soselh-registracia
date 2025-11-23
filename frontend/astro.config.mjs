// @ts-check
import {defineConfig} from "astro/config";

// https://astro.build/config
export default defineConfig({
    i18n: {
        locales: ["sk"],
        defaultLocale: "sk",
    },
    experimental: {
        svgo: true,
    }
});
