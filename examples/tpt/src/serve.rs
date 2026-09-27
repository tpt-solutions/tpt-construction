// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `tpt serve` — a thin, dependency-free HTTP interface over the takeoff,
//! estimate, and CPM scheduling engines.
//!
//! Enabled by the `serve` feature; the server itself is hand-rolled HTTP/1.1
//! over [`std::net`] so the CLI grows no runtime dependencies. Endpoints:
//!
//! ```text
//! GET  /health           liveness probe
//! POST /api/v1/takeoff   body: neutral model JSON -> takeoff quantities
//! POST /api/v1/estimate  body: {"model": …, "rates": …, "currency": "USD"} -> priced estimate
//! POST /api/v1/schedule  body: ScheduleNetwork JSON -> solved CPM result
//! ```
//!
//! This is a demo/evaluation surface: HTTP/1.0-style `Connection: close` per
//! request, no TLS, no keep-alive, JSON only. When started with `--token`,
//! requests must present `Authorization: Bearer <token>` and carry the scope
//! each endpoint requires (`read` for takeoff, `write` for estimate and
//! schedule; `admin` grants both). Every request is logged to stdout as a
//! [`tpt_c_api::AuditLogEntry`] JSON line.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::ExitCode;

use serde_json::{json, Value};
use tpt_c_api::{AuditLogEntry, AuthToken, Permission};
use tpt_c_cost::CostDatabase;
use tpt_c_estimating::EstimateBuilder;
use tpt_c_model::Project;
use tpt_c_quantities::{QuantityKind, TakeoffEngine, TakeoffResult};
use tpt_c_schedule::{CpmResult, ScheduleNetwork};

/// Refuse bodies larger than 10 MiB.
const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

/// Server settings shared across connections.
#[derive(Clone)]
pub(crate) struct ServerConfig {
    /// When set, requests must authenticate with this token and its scopes.
    pub token: Option<AuthToken>,
}

/// Result of an endpoint handler: `Ok` for expected answers, `Err` for
/// error statuses.
type ApiResult = Result<(u16, Value), (u16, Value)>;

/// Parse `tpt serve` arguments and run the accept loop.
pub(crate) fn run_cli(args: &[String]) -> ExitCode {
    let mut addr = String::from("127.0.0.1:8090");
    let mut token: Option<AuthToken> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--addr" => match args.get(i + 1) {
                Some(a) => {
                    addr = a.clone();
                    i += 1;
                }
                None => {
                    eprintln!("--addr requires a value");
                    return ExitCode::FAILURE;
                }
            },
            "--token" => match args.get(i + 1) {
                Some(t) => {
                    token = Some(
                        AuthToken::new(t.clone(), "cli", iso8601_now())
                            .with_scopes(vec!["read".into(), "write".into()]),
                    );
                    i += 1;
                }
                None => {
                    eprintln!("--token requires a value");
                    return ExitCode::FAILURE;
                }
            },
            other => {
                eprintln!("unexpected argument: {other}");
                eprintln!("usage: tpt serve [--addr <addr>] [--token <secret>]");
                return ExitCode::FAILURE;
            }
        }
        i += 1;
    }

    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cannot bind {addr}: {e}");
            return ExitCode::FAILURE;
        }
    };
    eprintln!("tpt serve listening on http://{addr} (Ctrl-C to stop)");
    if token.is_some() {
        eprintln!("bearer-token authentication enabled");
    }
    run(listener, ServerConfig { token })
}

/// Accept loop: one thread per connection, `Connection: close` semantics.
fn run(listener: TcpListener, cfg: ServerConfig) -> ExitCode {
    for stream in listener.incoming().flatten() {
        let cfg = cfg.clone();
        std::thread::spawn(move || {
            let mut stream = stream;
            let _ = handle_connection(&mut stream, &cfg);
        });
    }
    ExitCode::SUCCESS
}

/// Read one HTTP request, route it, write one response.
fn handle_connection(stream: &mut TcpStream, cfg: &ServerConfig) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let req = match read_request(&mut reader) {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(()),
        Err(status) => return write_response(stream, status, &json!({ "error": reason(status) })),
    };
    let (status, body) = route(cfg, &req);
    audit_log(cfg, &req, status);
    write_response(stream, status, &body)
}

/// One parsed HTTP request.
struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Read request head + body. `Ok(None)` means the peer closed cleanly.
fn read_request(reader: &mut BufReader<TcpStream>) -> Result<Option<Request>, u16> {
    let mut line = String::new();
    let n = reader.read_line(&mut line).map_err(|_| 400u16)?;
    if n == 0 || line.trim().is_empty() {
        return Ok(None);
    }
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or(400u16)?.to_ascii_uppercase();
    let target = parts.next().ok_or(400u16)?.to_string();
    let path = target.split('?').next().unwrap_or("/").to_string();

    let mut headers = Vec::new();
    loop {
        let mut h = String::new();
        let n = reader.read_line(&mut h).map_err(|_| 400u16)?;
        if n == 0 || h.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
        }
    }

    let len = headers
        .iter()
        .find(|(k, _)| k == "content-length")
        .and_then(|(_, v)| v.parse::<usize>().ok())
        .unwrap_or(0);
    if len > MAX_BODY_BYTES {
        return Err(413);
    }
    let mut body = vec![0u8; len];
    if len > 0 {
        reader.read_exact(&mut body).map_err(|_| 400u16)?;
    }
    Ok(Some(Request {
        method,
        path,
        headers,
        body,
    }))
}

/// Route a request to an endpoint handler.
fn route(cfg: &ServerConfig, req: &Request) -> (u16, Value) {
    let collapse = |r: ApiResult| match r {
        Ok((s, v)) | Err((s, v)) => (s, v),
    };
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/health") => (200, json!({ "status": "ok" })),
        ("POST", "/api/v1/takeoff") => {
            collapse(authorized(cfg, req, Permission::Read).and_then(|()| takeoff(req)))
        }
        ("POST", "/api/v1/estimate") => {
            collapse(authorized(cfg, req, Permission::Write).and_then(|()| estimate(req)))
        }
        ("POST", "/api/v1/schedule") => {
            collapse(authorized(cfg, req, Permission::Write).and_then(|()| schedule(req)))
        }
        ("GET", _) | ("POST", _) => (404, json!({ "error": "not found", "path": req.path })),
        _ => (405, json!({ "error": "method not allowed" })),
    }
}

/// Enforce bearer-token auth and per-endpoint scopes when a token is set.
fn authorized(cfg: &ServerConfig, req: &Request, needed: Permission) -> Result<(), (u16, Value)> {
    let Some(token) = &cfg.token else {
        return Ok(());
    };
    let supplied = req
        .header("authorization")
        .and_then(|v| v.strip_prefix("Bearer "));
    match supplied {
        Some(t) if t == token.token => {}
        _ => return Err((401, json!({ "error": "unauthorized" }))),
    }
    let needed = permission_slug(needed);
    if !token.scopes.is_empty() && !token.scopes.iter().any(|s| s == "admin" || s == needed) {
        return Err((
            403,
            json!({ "error": "forbidden", "required_scope": needed }),
        ));
    }
    Ok(())
}

fn permission_slug(p: Permission) -> &'static str {
    match p {
        Permission::Read => "read",
        Permission::Write => "write",
        Permission::Admin => "admin",
        Permission::Billing => "billing",
        Permission::Finance => "finance",
    }
}

/// POST /api/v1/takeoff: neutral model JSON -> quantity takeoff summary.
fn takeoff(req: &Request) -> ApiResult {
    let project: Project = serde_json::from_slice(&req.body)
        .map_err(|e| (400, json!({ "error": format!("invalid model JSON: {e}") })))?;
    let result = TakeoffEngine::new().run(&project);
    Ok((200, takeoff_summary(&project.name, &result)))
}

/// Shape the (non-`serde`) takeoff result into JSON.
fn takeoff_summary(project_name: &str, result: &TakeoffResult) -> Value {
    let items: Vec<Value> = result
        .items
        .iter()
        .map(|i| {
            json!({
                "element_id": i.element_id.to_string(),
                "name": i.name,
                "category": i.category,
                "quantities": i.quantities.iter().map(|q| json!({
                    "name": q.name,
                    "net": q.net.base_value(),
                    "gross": q.gross().base_value(),
                    "waste": q.waste.ratio(),
                    "manual": q.manual,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    let totals: Vec<Value> = [
        QuantityKind::Count,
        QuantityKind::Length,
        QuantityKind::Area,
        QuantityKind::Volume,
        QuantityKind::Mass,
    ]
    .iter()
    .map(|k| {
        json!({
            "kind": k,
            "net": result.total_net(*k),
            "gross": result.total_gross(*k),
        })
    })
    .collect();
    json!({
        "project": project_name,
        "elements": result.items.len(),
        "items": items,
        "totals": totals,
    })
}

/// POST /api/v1/estimate: model + rates -> priced estimate JSON.
fn estimate(req: &Request) -> ApiResult {
    let payload: Value = serde_json::from_slice(&req.body)
        .map_err(|e| (400, json!({ "error": format!("invalid JSON: {e}") })))?;
    let model: Project = serde_json::from_value(
        payload
            .get("model")
            .cloned()
            .ok_or((400, json!({ "error": "missing 'model'" })))?,
    )
    .map_err(|e| (400, json!({ "error": format!("invalid model: {e}") })))?;
    let rates: CostDatabase = match payload.get("rates") {
        Some(r) => serde_json::from_value(r.clone())
            .map_err(|e| (400, json!({ "error": format!("invalid rates: {e}") })))?,
        None => CostDatabase::new(),
    };
    let currency = payload
        .get("currency")
        .and_then(Value::as_str)
        .unwrap_or("USD")
        .to_string();

    let builder = EstimateBuilder::new(format!("Estimate — {}", model.name), &currency)
        .for_project(&model)
        .with_database(rates);
    let estimate = builder
        .from_project(&model)
        .map_err(|e| (422, json!({ "error": e.to_string() })))?;
    let body = serde_json::to_value(&estimate)
        .map_err(|e| (500, json!({ "error": format!("serialize estimate: {e}") })))?;
    Ok((200, body))
}

/// POST /api/v1/schedule: `ScheduleNetwork` JSON -> solved `CpmResult` JSON.
fn schedule(req: &Request) -> ApiResult {
    let network: ScheduleNetwork = serde_json::from_slice(&req.body).map_err(|e| {
        (
            400,
            json!({ "error": format!("invalid network JSON: {e}") }),
        )
    })?;
    let result: CpmResult = network
        .schedule()
        .map_err(|e| (422, json!({ "error": e.to_string() })))?;
    let body = serde_json::to_value(&result)
        .map_err(|e| (500, json!({ "error": format!("serialize result: {e}") })))?;
    Ok((200, body))
}

/// Emit one audit line per handled request.
fn audit_log(cfg: &ServerConfig, req: &Request, status: u16) {
    let actor = req
        .header("authorization")
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| match &cfg.token {
            Some(tok) if tok.token == t => tok.subject.clone(),
            _ => "unknown".to_string(),
        })
        .unwrap_or_else(|| "anonymous".to_string());
    let entry = AuditLogEntry::new(
        iso8601_now(),
        actor,
        format!("http.{} {}", req.method.to_lowercase(), req.path),
    )
    .with_payload(json!({ "status": status, "body_bytes": req.body.len() }));
    println!("{}", serde_json::to_string(&entry).unwrap_or_default());
}

/// Write a JSON HTTP/1.1 response with `Connection: close`.
fn write_response(stream: &mut TcpStream, status: u16, body: &Value) -> std::io::Result<()> {
    let payload = serde_json::to_vec(body).unwrap_or_else(|_| b"{}".to_vec());
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        status,
        reason(status),
        payload.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&payload)
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        422 => "Unprocessable Entity",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

/// Current UTC time as an ISO 8601 string (no external time crate).
fn iso8601_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Days-since-epoch to civil date (Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    use tpt_c_core::ActivityId;
    use tpt_c_schedule::{Activity, Dependency};
    use tpt_c_units::Duration;
    use uuid::Uuid;

    const GOLDEN_MODEL: &str = include_str!("../../../test-data/golden/sample-model.json");

    /// Serve connections from `listener` synchronously on a background thread.
    fn start_server(cfg: ServerConfig) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let _ = handle_connection(&mut stream, &cfg);
            }
        });
        addr
    }

    fn exchange(addr: SocketAddr, raw: &str) -> String {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(raw.as_bytes()).unwrap();
        let mut buf = String::new();
        let _ = stream.read_to_string(&mut buf);
        buf
    }

    fn post(addr: SocketAddr, path: &str, body: &Value, token: Option<&str>) -> String {
        let payload = body.to_string();
        let auth = token
            .map(|t| format!("Authorization: Bearer {t}\r\n"))
            .unwrap_or_default();
        let raw = format!(
            "POST {path} HTTP/1.1\r\nHost: test\r\n{auth}Content-Type: application/json\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{payload}",
            payload.len()
        );
        exchange(addr, &raw)
    }

    fn status_of(response: &str) -> u16 {
        response
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    fn body_of(response: &str) -> Value {
        let (_, rest) = response.split_once("\r\n\r\n").unwrap();
        serde_json::from_str(rest.trim()).unwrap_or(Value::Null)
    }

    fn two_activity_network() -> Value {
        let a = ActivityId::from_uuid(Uuid::from_u128(1));
        let b = ActivityId::from_uuid(Uuid::from_u128(2));
        let mut net = ScheduleNetwork::new();
        net.add_activity(Activity::new(a, "Dig", Duration::from_hours(8.0)))
            .unwrap();
        net.add_activity(Activity::new(b, "Pour", Duration::from_hours(16.0)))
            .unwrap();
        net.add_dependency(Dependency::finish_to_start(a, b))
            .unwrap();
        serde_json::to_value(&net).unwrap()
    }

    #[test]
    fn health_and_unknown_routes() {
        let addr = start_server(ServerConfig { token: None });
        let resp = exchange(
            addr,
            "GET /health HTTP/1.1\r\nHost: t\r\nConnection: close\r\n\r\n",
        );
        assert_eq!(status_of(&resp), 200);
        assert!(resp.contains("\"ok\""));

        let resp = exchange(
            addr,
            "GET /api/v1/nope HTTP/1.1\r\nHost: t\r\nConnection: close\r\n\r\n",
        );
        assert_eq!(status_of(&resp), 404);

        let resp = exchange(
            addr,
            "DELETE /health HTTP/1.1\r\nHost: t\r\nConnection: close\r\n\r\n",
        );
        assert_eq!(status_of(&resp), 405);
    }

    #[test]
    fn takeoff_over_http() {
        let addr = start_server(ServerConfig { token: None });
        let model: Value = serde_json::from_str(GOLDEN_MODEL).unwrap();
        let resp = post(addr, "/api/v1/takeoff", &model, None);
        assert_eq!(status_of(&resp), 200, "response: {resp}");
        let body = body_of(&resp);
        assert!(body["elements"].as_u64().unwrap_or(0) > 0);
        assert!(body["totals"].as_array().unwrap().len() == 5);
    }

    #[test]
    fn estimate_over_http() {
        let addr = start_server(ServerConfig { token: None });
        let csv = std::fs::read_to_string("../../test-data/golden/rates.csv").unwrap();
        let db: CostDatabase =
            tpt_c_csv::read_cost_database(std::io::Cursor::new(csv.into_bytes())).unwrap();
        let body = json!({
            "model": serde_json::from_str::<Value>(GOLDEN_MODEL).unwrap(),
            "rates": serde_json::to_value(&db).unwrap(),
            "currency": "USD",
        });
        let resp = post(addr, "/api/v1/estimate", &body, None);
        assert_eq!(status_of(&resp), 200, "response: {resp}");
        let body = body_of(&resp);
        assert!(
            !body["line_items"]
                .as_array()
                .unwrap_or(&Vec::new())
                .is_empty(),
            "expected at least one line item"
        );
    }

    #[test]
    fn schedule_over_http() {
        let addr = start_server(ServerConfig { token: None });
        let network = two_activity_network();
        let resp = post(addr, "/api/v1/schedule", &network, None);
        assert_eq!(status_of(&resp), 200, "response: {resp}");
        let body = body_of(&resp);
        assert_eq!(body["project_duration"].as_f64(), Some(24.0));
        assert_eq!(body["critical_path"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn token_auth_and_scopes() {
        let token = AuthToken::new("s3cret", "field-device", iso8601_now())
            .with_scopes(vec!["read".into()]);
        let addr = start_server(ServerConfig { token: Some(token) });

        // Unauthenticated requests are rejected.
        let resp = post(addr, "/api/v1/takeoff", &json!({}), None);
        assert_eq!(status_of(&resp), 401);

        // Read-scoped token may take off but not price estimates.
        let model: Value = serde_json::from_str(GOLDEN_MODEL).unwrap();
        let resp = post(addr, "/api/v1/takeoff", &model, Some("s3cret"));
        assert_eq!(status_of(&resp), 200);

        let csv = std::fs::read_to_string("../../test-data/golden/rates.csv").unwrap();
        let db: CostDatabase =
            tpt_c_csv::read_cost_database(std::io::Cursor::new(csv.into_bytes())).unwrap();
        let estimate_body = json!({
            "model": serde_json::from_str::<Value>(GOLDEN_MODEL).unwrap(),
            "rates": serde_json::to_value(&db).unwrap(),
        });
        let resp = post(addr, "/api/v1/estimate", &estimate_body, Some("s3cret"));
        assert_eq!(status_of(&resp), 403);
    }
}
