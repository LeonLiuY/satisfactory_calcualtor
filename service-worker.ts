
const CACHE_NAME = "v1";
const ALWAYS_FETCH_PATHS: string[] = ["/", "/index.html", "/service-worker.js"];
const ALWAYS_CACHE_EXTS: string[] = [".js", ".css", ".wasm"];

self.addEventListener("fetch", (event: FetchEvent) => {
  const url = new URL(event.request.url);
  if (ALWAYS_FETCH_PATHS.includes(url.pathname)) {
    event.respondWith(
      fetch(event.request)
        .then(response => {
          caches.open(CACHE_NAME).then(cache => {
            cache.put(event.request, response)
          });
          return response.clone();
        })
        .catch(() => caches.match(event.request))
    );
  } else if (ALWAYS_CACHE_EXTS.some(ext => url.pathname.endsWith(ext))) {
    event.respondWith(
      caches.match(event.request).then(response => {
        return response || fetch(event.request).then(fetchRes => {
          caches.open(CACHE_NAME).then(cache => cache.put(event.request, fetchRes));
          return fetchRes.clone();
        })
      })
    );
  }
});
