# StubHouse Test API Server

This is a local mock server designed to test all features of StubHouse.

## How to Run

Ensure you have [Bun](https://bun.sh) installed.

```bash
bun test-api-server/server.ts
```

The server will be available at `http://localhost:3000`.

## Test Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/get` | GET | Returns request details (headers, query params). |
| `/post` | POST | Returns request details and body (JSON, Text, or Form). |
| `/put` | PUT | Same as `/post`. |
| `/patch` | PATCH | Same as `/post`. |
| `/delete` | DELETE | Same as `/post`. |
| `/echo` | ANY | Returns exactly what was sent in a JSON format. |
| `/auth/bearer` | ANY | Requires `Authorization: Bearer <token>` header. |
| `/auth/basic` | ANY | Requires `Authorization: Basic <credentials>` header. |
| `/auth/apikey/header` | ANY | Requires `X-API-Key` header. |
| `/auth/apikey/query` | ANY | Requires `api_key` query parameter. |
| `/status/:code` | ANY | Returns the specified HTTP status code (e.g., `/status/404`). |
| `/delay/:ms` | ANY | Delays the response by `:ms` milliseconds (e.g., `/delay/2000`). |

## Usage in StubHouse

Simply enter `http://localhost:3000/get` (or any other endpoint) in the URL bar of the StubHouse GUI.
