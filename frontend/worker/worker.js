const CONFIG_PATH = "/config.json";

export default {
    async fetch(request, env) {
        const url = new URL(request.url);

        if (url.pathname === CONFIG_PATH) {
            const backendUrl = env.BACKEND_URL;
            if (!backendUrl) {
                return new Response(
                    JSON.stringify({ error: "BACKEND_URL is not configured" }),
                    { status: 500 },
                );
            }

            return new Response(
                JSON.stringify({ backend_url: backendUrl }),
            );
        }

        try {
            return await env.ASSETS.fetch(request);
        } catch (error) {
            console.error("Static asset fetch failed", error);
            return new Response("Not Found", { status: 404 });
        }
    },
};
