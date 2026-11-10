import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// Ports come from the root .env (exported by start.sh); the defaults match .env.example.
const backend = `http://${process.env.BIND_ADDR ?? '127.0.0.1:8090'}`

export default defineConfig({
  plugins: [react()],
  server: {
    port: Number(process.env.FRONTEND_PORT ?? 5180),
    strictPort: true,
    proxy: { '/api': backend },
  },
})
