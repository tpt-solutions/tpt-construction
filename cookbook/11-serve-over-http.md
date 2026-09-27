# 11. Serve the engines over HTTP

> **Crates used:** the `tpt` CLI with its `serve` feature (uses `tpt-c-api` types for auth and audit)

`tpt serve` exposes the takeoff, estimate, and CPM scheduling engines behind
a thin, dependency-free HTTP/1.1 interface — useful for driving the engines
from a browser, a script, or a field tablet app. It is a demo/evaluation
surface: `Connection: close` per request, JSON in/out, no TLS.

## Build and run

The server is feature-gated so default builds stay lean:

```bash
cargo build -p tpt --features serve
cargo run -p tpt -- serve --addr 127.0.0.1:8090 --token s3cret
```

```text
tpt serve listening on http://127.0.0.1:8090 (Ctrl-C to stop)
bearer-token authentication enabled
```

Without `--token` the endpoints are open (handy for local testing).

## Endpoints

| Method | Path | Body | Result |
|---|---|---|---|
| GET | `/health` | — | `{"status":"ok"}` |
| POST | `/api/v1/takeoff` | neutral model JSON (recipe 1) | per-element net/gross quantities + kind totals |
| POST | `/api/v1/estimate` | `{"model": …, "rates": …, "currency": "USD"}` | the priced estimate as JSON |
| POST | `/api/v1/schedule` | `ScheduleNetwork` JSON | solved `CpmResult` (floats, critical path) |

Scopes: takeoff needs `read`; estimate and schedule need `write` (`admin`
grants both). Every request prints an `AuditLogEntry` JSON line to stdout.

## Try it

Health check:

```bash
curl -s http://127.0.0.1:8090/health
```

```json
{"status":"ok"}
```

Solve a schedule (two-activity chain, 8 h + 16 h = 24 working hours):

```bash
curl -s -X POST http://127.0.0.1:8090/api/v1/schedule \
  -H "Authorization: Bearer s3cret" \
  -d '{"activities":{"<uuid-1>":{"id":"<uuid-1>","name":"Dig","duration":{"hours":8.0}}},
       "deps":[{"predecessor":"<uuid-1>","successor":"<uuid-2>","relationship":"finish_to_start","lag_hours":0.0}],
       "constraints":[]}'
```

The response carries per-activity floats and the critical path:

```json
{"activities":{...},"project_duration":24.0,"critical_path":["<uuid-1>","<uuid-2>"]}
```

Price the golden sample model with the golden rate database:

```bash
python - <<'EOF'
import json, urllib.request
model = json.load(open("test-data/golden/sample-model.json"))
body = json.dumps({"model": model, "currency": "USD"}).encode()
req = urllib.request.Request(
    "http://127.0.0.1:8090/api/v1/estimate", data=body,
    headers={"Content-Type": "application/json",
             "Authorization": "Bearer s3cret"})
result = json.load(urllib.request.urlopen(req))
print(result["title"], "->", len(result["line_items"]), "line items")
EOF
```

## Notes

- Auth failures return `401` (missing/wrong bearer) or `403` (token lacks
  the scope); bad payloads return `400`; engine rejections such as a missing
  rate or a cyclic network return `422` with the engine's error message.
- Responses always carry `Connection: close` and permissive CORS so browser
  demos (see `examples/wasm-browser-demo`) can call them.
- The `estimate` endpoint's `rates` value is a serialized `CostDatabase`
  (map of code → rate). Load your CSV database and send it along, or omit
  `rates` to get a `422` listing the first missing code.
- The same request/response shapes work from `fetch()` in the browser.
