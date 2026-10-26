import { createMcpHandler } from "agents/mcp/server";
import { createServer, type Env } from "./mcp";
const MAX_REQUEST_BYTES = 2 * 1024 * 1024;
async function bounded(request: Request): Promise<Request | Response> {
  if (!request.body) return request;
  const reader = request.body.getReader(),
    chunks: Uint8Array[] = [];
  let size = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    size += value.byteLength;
    if (size > MAX_REQUEST_BYTES) {
      await reader.cancel();
      return new Response("Request too large", { status: 413 });
    }
    chunks.push(value);
  }
  const bytes = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return new Request(request.url, {
    method: request.method,
    headers: request.headers,
    body: bytes,
  });
}
function cors(response: Response): Response {
  const result = new Response(response.body, response);
  result.headers.set("Access-Control-Allow-Origin", "*");
  result.headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
  result.headers.set(
    "Access-Control-Allow-Headers",
    "Content-Type, Accept, MCP-Protocol-Version, MCP-Session-Id, MCP-Method, MCP-Name, Last-Event-ID",
  );
  result.headers.set(
    "Access-Control-Expose-Headers",
    "MCP-Session-Id, MCP-Protocol-Version",
  );
  return result;
}
export default {
  async fetch(
    request: Request,
    env: Env,
    ctx: ExecutionContext,
  ): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname === "/health")
      return Response.json({ status: "ok", version: "0.1.0" });
    if (url.pathname === "/")
      return new Response(
        "Popup MCP App\nConnect your MCP Apps client to /mcp.\nForms use progressive disclosure; submissions continue your conversation.\n",
        { headers: { "Content-Type": "text/plain; charset=utf-8" } },
      );
    if (url.pathname !== "/mcp")
      return new Response("Not found", { status: 404 });
    if (request.method === "OPTIONS")
      return cors(new Response(null, { status: 204 }));
    const limited = await bounded(request);
    if (limited instanceof Response) return cors(limited);
    return cors(
      await createMcpHandler(() => createServer(env))(limited, env, ctx),
    );
  },
} satisfies ExportedHandler<Env>;
