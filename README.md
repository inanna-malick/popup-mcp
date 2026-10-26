# Popup

**Live, structured interfaces for agent conversations.** Popup lets an agent anticipate likely replies and their follow-ups, then present them together as an interactive MCP App. Explore compatible directions, adjust sliders and checkboxes, edit proposals, and send your answers back in one submission.

The main app runs on **Cloudflare Workers**, with a compact Solid/TypeScript interface inside your conversation. It carries forward the original popup-common form DSL: conditional subsections respond locally as you make choices.

## Connect

Add a remote MCP server in a client that supports **MCP Apps**:

| Setting | Value |
| --- | --- |
| Name | Popup |
| Server URL | `https://popup-mcp-app.inanna-c38.workers.dev/mcp` |
| Authentication | None |

Ask your agent to use `popup` to explore a decision, compare approaches, or work through a proposal with conditional follow-ups. The interface appears inline; submitting returns your selected answers to the conversation.

The Worker stores no forms or answers. Open drafts live in the app iframe, and submitted answers become messages in your host conversation.

## Run or deploy your own

Use Node 24.11+ or Node 22.18+, and npm:

```sh
git clone https://github.com/inanna-malick/popup-mcp.git
cd popup-mcp/apps/popup
npm ci
npm run dev
```

The local MCP endpoint is `http://localhost:8787/mcp`. To deploy to your Cloudflare account, choose a Worker name in `wrangler.jsonc` and run:

```sh
npm run deploy
```

No database, Durable Object, model API, or secret is required.

See the **[MCP App documentation](apps/popup/README.md)** for the DSL and submission contract, development checks, deployment details, and screenshot-based visual development loop.

## Native Rust app

The original desktop app remains available for native popup windows and local stdio MCP connections:

```sh
cargo install --path crates/popup-gui
popup --file examples/simple_confirm.json
```

For a local MCP connection, configure your client to launch the installed `popup` executable without arguments. Use its absolute path when your client does not inherit your shell's PATH.

The Rust implementation also contains experimental semantic routing and conversation trees. These are separate from the Cloudflare app, which implements the classic form DSL and deterministic conditional branches.

## Background and contributing

[Stop using natural language interfaces](https://tidepool.leaflet.pub/3mcbegnuf2k2i) describes the conversational motivation. See [apps/popup](apps/popup/README.md) for the current app and [CLAUDE.md](CLAUDE.md) for native implementation guidance.

## License

MIT
