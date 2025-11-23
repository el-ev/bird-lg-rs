export default {
  async fetch(request, env) {
    try {
      return await env.ASSETS.fetch(request);
    } catch (error) {
      console.error("Static asset fetch failed", error);
      return new Response("Not Found", { status: 404 });
    }
  },
};