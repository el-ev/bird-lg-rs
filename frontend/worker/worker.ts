/// <reference path="../.wrangler/types.d.ts" />

const CONFIG_PATH = "/config.json";

const jsonResponse = (body: unknown, init: ResponseInit = {}): Response => {
    const headers = new Headers(init.headers ?? {});
    if (!headers.has("content-type")) {
        headers.set("content-type", "application/json");
    }

    return new Response(JSON.stringify(body), { ...init, headers });
};

export default {
    async fetch(request: Request, env: Env): Promise<Response> {
        const url = new URL(request.url);

        if (url.pathname === CONFIG_PATH) {
            const backendUrl = env.BACKEND_URL;
            const username = env.USERNAME ?? null;

            if (!backendUrl) {
                return jsonResponse(
                    { error: "BACKEND_URL is not configured" },
                    { status: 500 },
                );
            }

            return jsonResponse({ backend_url: backendUrl, username });
        }

        try {
            const assetResponse = await env.ASSETS.fetch(request);
            if (assetResponse.status === 404) {
                const indexUrl = new URL("/index.html", request.url).toString();
                const fallbackResponse = await env.ASSETS.fetch(indexUrl);
                if (fallbackResponse.status === 200) {
                    return fallbackResponse;
                }
            }
            return assetResponse;
        } catch (error) {
            console.error("Static asset fetch failed", error);
            return new Response("Not Found", { status: 404 });
        }
    },
};
