const CACHE_NAME = 'invoicer-v1';
const SHARED_CACHE = 'invoicer-shared-v1';

const STATIC_ASSETS = [
  './',
  './index.html',
  './style.css',
  './app.js',
  './manifest.json',
  './icons/icon.svg',
  './pkg/invoicer.js',
  './pkg/invoicer_bg.wasm'
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(STATIC_ASSETS))
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys
          .filter((k) => k !== CACHE_NAME && k !== SHARED_CACHE)
          .map((k) => caches.delete(k))
      )
    )
  );
  self.clients.claim();
});

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);

  // Intercept Web Share Target POST request from Android Gmail / files
  if (event.request.method === 'POST' && url.pathname.endsWith('share-target')) {
    event.respondWith(
      (async () => {
        try {
          const formData = await event.request.formData();
          const file = formData.get('invoice') || formData.get('file');

          if (file) {
            const cache = await caches.open(SHARED_CACHE);
            await cache.put('shared-invoice-pdf', new Response(file, {
              headers: { 'Content-Type': file.type || 'application/pdf' }
            }));
            return Response.redirect('./index.html?shared=1', 303);
          }
        } catch (err) {
          console.error('Failed to process shared file:', err);
        }
        return Response.redirect('./index.html', 303);
      })()
    );
    return;
  }

  // Cache first with network fallback for other requests
  event.respondWith(
    caches.match(event.request).then((cached) => {
      if (cached) {
        return cached;
      }
      return fetch(event.request).catch(() => {
        if (event.request.destination === 'document') {
          return caches.match('./index.html');
        }
      });
    })
  );
});
