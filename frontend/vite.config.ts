import react from '@vitejs/plugin-react'
import { defineConfig, loadEnv } from 'vite'

// Settings come from frontend/.env (see .env.example) or the environment, which wins:
// start.sh passes the backend's real address, so the two cannot drift apart.
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, import.meta.dirname, '')
  return {
    plugins: [react()],
    server: {
      port: Number(env.FRONTEND_PORT || 5180),
      strictPort: true,
      proxy: { '/api': env.BACKEND_URL || 'http://127.0.0.1:8090' },
    },
  }
})
