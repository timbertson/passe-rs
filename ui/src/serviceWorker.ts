const DEBUG_VERSION = 3
const CACHE_KEY = 'v1'

function claimClients() {
	console.log('claiming clients');
	(self as any).clients.claim();
}

async function dumpCacheContents() {
	console.log('cache dump:');
	const cache = await caches.open(CACHE_KEY);
	console.log('cache keys: ', await cache.keys());
}

self.addEventListener('message', async (event) => {
	const message = event.data;
	console.log(`Service worker v${DEBUG_VERSION} saw ${message}`);

	if (message == 'loaded') {
		claimClients();
	} else if (message == 'dump') {
		await dumpCacheContents();
	} else {
		console.error(`unknown message: ${message}`);
	}
});

self.addEventListener("activate", async (event: Event) => {
	claimClients();
});

const notFound = new Response("Network error", {
	status: 404,
	headers: { "Content-Type": "text/plain" },
});

const fetchRequest = async (request: Request) => {
	if (request.method != 'GET') {
		return await fetch(request); // don't intercept
	} else {
		const cache = await caches.open(CACHE_KEY);
		try {
			const response = await fetch(request.clone());
			if (response.status >= 200 && response.status < 400) {
				console.log(`Caching ${request.url} response`, response)
				await cache.put(request.clone(), response.clone());
			} else {
				console.log(`Not caching ${request.url} response (${response.status})`, response)
			}
			return response;
		} catch (e) {
			console.log("Network error (will check cache):", e);
			const cached = await cache.match(request);
			if (!cached) {
				console.warn("No cache entry for request", request);
			}
			console.log("Returning cached:", cached);
			return cached || notFound;
		}
	}
};

self.addEventListener('fetch', (event) => {
	const fetchEvent = event as any;
	fetchEvent.respondWith(fetchRequest(fetchEvent.request));
});
