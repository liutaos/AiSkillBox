// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  base: './',
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:10882',
        changeOrigin: true
      }
    }
  }
})
