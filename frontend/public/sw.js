// Keeps the app's own files so it opens instantly and without a connection.
// Prices are never kept here: they come from the backend, on another address.
const CACHE = 'ieftin-shell-v1'
const PAGE = '/index.html'

self.addEventListener('install', (event) => {
  event.waitUntil(caches.open(CACHE).then((cache) => cache.add(PAGE)))
  self.skipWaiting()
})

self.addEventListener('activate', (event) => {
  const dropOld = async () => {
    const names = await caches.keys()
    await Promise.all(names.filter((name) => name !== CACHE).map((name) => caches.delete(name)))
    await self.clients.claim()
  }
  event.waitUntil(dropOld())
})

self.addEventListener('fetch', (event) => {
  const { request } = event
  const url = new URL(request.url)
  if (request.method !== 'GET' || url.origin !== self.location.origin || url.pathname.startsWith('/api/')) return

  if (request.mode === 'navigate') {
    // Every route is the same page. Prefer a fresh copy; offline, use the kept one.
    event.respondWith(fresh(PAGE).catch(() => caches.match(PAGE)))
  } else if (url.pathname.startsWith('/assets/')) {
    // Content-hashed by the build: a name never changes its content.
    event.respondWith(caches.match(request).then((kept) => kept ?? fresh(request)))
  } else {
    event.respondWith(fresh(request).catch(() => caches.match(request)))
  }
})

/** Fetches from the network and keeps a copy of a good answer. */
async function fresh(request) {
  const response = await fetch(request)
  if (response.ok) {
    const cache = await caches.open(CACHE)
    await cache.put(request, response.clone())
  }
  return response
}
