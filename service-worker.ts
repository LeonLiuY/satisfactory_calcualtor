
const CACHE_NAME = "v1";
const ALWAYS_FETCH_PATHS: string[] = ["/", "/index.html", "/service-worker.js"];
const ALWAYS_CACHE_EXTS: string[] = [".js", ".css", ".wasm"];

self.addEventListener("fetch", (event: FetchEvent) => {
  const url = new URL(event.request.url);
  if (ALWAYS_FETCH_PATHS.includes(url.pathname)) {
    console.log(`fetch ${url.pathname}`)
    event.respondWith(
      fetch(event.request)
        .then(response => {
          console.log("")
          caches.open(CACHE_NAME).then(cache => {
            console.log(`cache ${url.pathname}`);
            cache.put(event.request, response)
          });
          return response.clone();
        })
        .catch(() => caches.match(event.request))
    );
  } else if (ALWAYS_CACHE_EXTS.some(ext => url.pathname.endsWith(ext))) {
    console.log(`match ${url.pathname}`)
    event.respondWith(
      caches.match(event.request).then(response => {
        console.log(`match ${response}`);
        return response || fetch(event.request).then(fetchRes => {
          caches.open(CACHE_NAME).then(cache => cache.put(event.request, fetchRes));
          return fetchRes.clone();
        })
      })
    );
  }
});
