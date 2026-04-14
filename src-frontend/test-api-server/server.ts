import { serve } from "bun";

const PORT = 3000;

console.log(`Test API Server running at http://localhost:${PORT}`);

const server = serve({
  port: PORT,
  async fetch(req) {
    const url = new URL(req.url);
    const method = req.method;
    const headers = Object.fromEntries(req.headers.entries());
    const query = Object.fromEntries(url.searchParams.entries());

    // Basic routing
    const path = url.pathname;

    // Helper to read body
    async function getBody() {
      const contentType = req.headers.get("content-type") || "";
      if (contentType.includes("application/json")) {
        try {
          return await req.json();
        } catch {
          return null;
        }
      } else if (contentType.includes("application/x-www-form-urlencoded")) {
        const formData = await req.formData();
        return Object.fromEntries(formData.entries());
      } else {
        return await req.text();
      }
    }

    const responseData = {
      method,
      url: req.url,
      path,
      headers,
      query,
      body: null as any,
    };

    if (["POST", "PUT", "PATCH", "DELETE"].includes(method)) {
      responseData.body = await getBody();
    }

    // --- Endpoints ---

    // Root: List all endpoints
    if (path === "/") {
      return new Response(`
        <html>
          <head>
            <style>
              body { font-family: sans-serif; padding: 20px; line-height: 1.6; background: #111; color: #eee; }
              code { background: #222; padding: 2px 4px; border-radius: 4px; color: #61afef; }
              h1 { color: #fff; }
              ul { list-style: none; padding: 0; }
              li { margin-bottom: 10px; border-bottom: 1px solid #333; padding-bottom: 10px; }
              .method { font-weight: bold; color: #c678dd; }
            </style>
          </head>
          <body>
            <h1>StubHouse Test API Server</h1>
            <p>Available endpoints for testing:</p>
            <ul>
              <li><span class="method">GET</span> <code>/get</code> - Returns request details (headers, params).</li>
              <li><span class="method">POST</span> <code>/post</code> - Returns request details and body.</li>
              <li><span class="method">PUT/PATCH/DELETE</span> <code>/put</code>, <code>/patch</code>, <code>/delete</code> - Returns request details and body.</li>
              <li><span class="method">ANY</span> <code>/echo</code> - Returns exactly what was sent.</li>
              <li><span class="method">GET</span> <code>/auth/bearer</code> - Requires Bearer token (any value).</li>
              <li><span class="method">GET</span> <code>/auth/basic</code> - Requires Basic Auth (any value).</li>
              <li><span class="method">GET</span> <code>/auth/apikey/header</code> - Requires <code>X-API-Key</code> header.</li>
              <li><span class="method">GET</span> <code>/auth/apikey/query</code> - Requires <code>api_key</code> query param.</li>
              <li><span class="method">GET</span> <code>/status/:code</code> - Returns specified status code (e.g., <code>/status/404</code>).</li>
              <li><span class="method">GET</span> <code>/delay/:ms</code> - Delays response by :ms milliseconds (e.g., <code>/delay/1000</code>).</li>
            </ul>
          </body>
        </html>
      `, { headers: { "Content-Type": "text/html" } });
    }

    if (path === "/get" || path === "/post" || path === "/put" || path === "/patch" || path === "/delete" || path === "/echo") {
      return Response.json(responseData, {
        headers: { "Access-Control-Allow-Origin": "*" }
      });
    }

    if (path === "/auth/bearer") {
      const auth = req.headers.get("Authorization");
      if (auth?.startsWith("Bearer ")) {
        return Response.json({ authenticated: true, token: auth.split(" ")[1] }, { headers: { "Access-Control-Allow-Origin": "*" } });
      }
      return new Response("Unauthorized: Bearer token required", { status: 401, headers: { "Access-Control-Allow-Origin": "*" } });
    }

    if (path === "/auth/basic") {
      const auth = req.headers.get("Authorization");
      if (auth?.startsWith("Basic ")) {
        return Response.json({ authenticated: true, credentials: auth.split(" ")[1] }, { headers: { "Access-Control-Allow-Origin": "*" } });
      }
      return new Response("Unauthorized: Basic auth required", { status: 401, headers: { "Access-Control-Allow-Origin": "*" } });
    }

    if (path === "/auth/apikey/header") {
      const key = req.headers.get("X-API-Key");
      if (key) {
        return Response.json({ authenticated: true, key }, { headers: { "Access-Control-Allow-Origin": "*" } });
      }
      return new Response("Unauthorized: X-API-Key header required", { status: 401, headers: { "Access-Control-Allow-Origin": "*" } });
    }

    if (path === "/auth/apikey/query") {
      const key = url.searchParams.get("api_key");
      if (key) {
        return Response.json({ authenticated: true, key }, { headers: { "Access-Control-Allow-Origin": "*" } });
      }
      return new Response("Unauthorized: api_key query param required", { status: 401, headers: { "Access-Control-Allow-Origin": "*" } });
    }

    if (path.startsWith("/status/")) {
      const code = parseInt(path.split("/").pop() || "200");
      return new Response(`Status code: ${code}`, { status: code, headers: { "Access-Control-Allow-Origin": "*" } });
    }

    if (path.startsWith("/delay/")) {
      const ms = parseInt(path.split("/").pop() || "0");
      await new Promise(resolve => setTimeout(resolve, ms));
      return Response.json({ delayed: true, ms }, { headers: { "Access-Control-Allow-Origin": "*" } });
    }

    return new Response("Not Found", { status: 404, headers: { "Access-Control-Allow-Origin": "*" } });
  },
});
