
const CACHE_NAME = "v1";
const ALWAYS_FETCH_PATHS: string[] = ["/", "/index.html"];

self.addEventListener("fetch", (event: FetchEvent) => {
  const url = new URL(event.request.url);
  if (ALWAYS_FETCH_PATHS.includes(url.pathname)) {
    event.respondWith(
      fetch(event.request)
        .then(response => {
          caches.open(CACHE_NAME).then(cache => cache.put(event.request, response.clone()));
          return response;
        })
        .catch(() => caches.match(event.request))
    );
  } else {
    event.respondWith(
      caches.match(event.request).then(response =>
        response || fetch(event.request).then(fetchRes => {
          caches.open(CACHE_NAME).then(cache => cache.put(event.request, fetchRes.clone()));
          return fetchRes;
        })
      )
    );
  }
});
