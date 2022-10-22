import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: "../www",
    emptyOutDir: true
  },
  css: {
    postcss: "./postcss.config.cjs"
  }
})
