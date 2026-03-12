# Kế hoạch tối ưu backend Rust (server-rs)

**Mục đích:** Cấu hình và tối ưu backend Rust cho môi trường dev và production, dễ vận hành và mở rộng sau này.

**Trạng thái:** Phase 1 đã triển khai (config, DB pool, scheduler). Các phase sau chờ triển khai.

---

## 1. Tổng quan ưu tiên

| Ưu tiên | Hạng mục | Mục tiêu | Effort ước lượng |
|--------|----------|----------|-------------------|
| P0 | Cấu hình hóa (config/env) | Pool, scheduler, timeouts đọc từ env; không hardcode | 1–2 ngày |
| P0 | Database pool | Số kết nối, timeout có thể cấu hình; phù hợp production | 0.5 ngày |
| P1 | HTTP layer | Giới hạn body, nén response, keep-alive | 0.5–1 ngày |
| P1 | Runner / adapter | Giới hạn concurrent runs (tùy chọn), timeout HTTP client | 0.5 ngày |
| P2 | Scheduler | Interval + backoff/jitter có cấu hình | 0.5 ngày |
| P2 | Build release | Profile release (LTO, strip), binary nhỏ gọn | 0.5 ngày |
| P3 | Observability | Request ID, structured log, (sau này) metrics | 1 ngày |
| P3 | Bảo mật | CORS theo env, security headers, giới hạn request size | 0.5 ngày |

---

## 2. Chi tiết từng hạng mục

### 2.1 Cấu hình hóa (Config / Env)

**Hiện trạng:** `config.rs` đọc `HOST`, `PORT`, `DATABASE_URL`, `UI_DIST`. Pool và scheduler dùng giá trị cố định (max_connections=10, interval 60s).

**Đề xuất:**

- Mở rộng `Config` với các biến môi trường (có default hợp lý):

  | Biến | Mặc định | Mô tả |
  |------|----------|--------|
  | `HOST` | `127.0.0.1` | Bind host |
  | `PORT` | `3100` | Bind port |
  | `DATABASE_URL` | (none) | PostgreSQL connection string |
  | `UI_DIST` | (auto ../ui/dist) | Thư mục static UI |
  | `DB_POOL_MAX_SIZE` | `10` | Số kết nối tối đa trong pool |
  | `DB_POOL_ACQUIRE_TIMEOUT_SECS` | `5` | Timeout lấy connection |
  | `DB_POOL_IDLE_TIMEOUT_SECS` | (optional) | Trả connection idle về pool |
  | `SCHEDULER_INTERVAL_SECS` | `60` | Chu kỳ heartbeat scheduler (giây) |
  | `RUNNER_MAX_CONCURRENT_RUNS` | `0` = không giới hạn | Số run adapter chạy đồng thời tối đa (0 = unlimited) |
  | `HTTP_BODY_MAX_BYTES` | `2 * 1024 * 1024` (2 MiB) | Giới hạn kích thước body request |
  | `RUST_LOG` | `info,tower_http=debug` | Log level (đã có, chỉ document) |

- Không bắt buộc file config: giữ ưu tiên env (phù hợp 12-factor, Docker/K8s). Sau này có thể thêm đọc từ file (e.g. `CONFIG_FILE`) nếu cần.

**Rủi ro:** Thấp. Chỉ thêm env và đọc trong `Config::from_env()` + `db::create_pool(&config)`.

---

### 2.2 Database pool

**Hiện trạng:** `db.rs` dùng `PgPoolOptions::new().max_connections(10).acquire_timeout(5s)`.

**Đề xuất:**

- Đọc từ config: `max_connections`, `acquire_timeout`, (optional) `idle_timeout`.
- Công thức tham chiếu: `max_connections` ≈ số worker/thread phục vụ request + vài kết nối cho scheduler/runner. Ví dụ: 1 process, ~10 request đồng thời → 10–20 là đủ; production có thể 20–50 tùy DB.
- Giữ `acquire_timeout` 5s; có thể thêm `idle_timeout` (ví dụ 600s) để tránh connection bị server đóng khi idle lâu.

**Rủi ro:** Thấp. Chỉ thay số cứng bằng config.

---

### 2.3 HTTP layer (Axum / Tower)

**Hiện trạng:** Không giới hạn body, không nén response, CORS cho phép Any.

**Đề xuất:**

- **Body size limit:** Dùng `axum::body::DefaultBodyLimit` (hoặc tương đương) với giá trị từ config `HTTP_BODY_MAX_BYTES`. Tránh request body quá lớn làm tốn bộ nhớ.
- **Compression (gzip):** Thêm `tower_http::compression` (hoặc tương đương) cho response JSON/HTML. Giảm băng thông, đặc biệt cho API trả nhiều dữ liệu.
- **Keep-alive:** Mặc định Axum/tokio thường đã bật; chỉ cần đảm bảo không tắt (không cần thay đổi code nếu hiện tại đã ổn).
- **Request timeout (global):** Tùy chọn: middleware timeout cho toàn bộ request (e.g. 30s–60s) để tránh request treo lâu. Có thể làm phase 2.

**Rủi ro:** Trung bình. Cần test kỹ compression với client (Accept-Encoding); body limit phải đủ lớn cho upload (assets, import).

---

### 2.4 Runner (adapter execution)

**Hiện trạng:** Mỗi run được `tokio::spawn` không giới hạn; HTTP client dùng timeout từ adapter config.

**Đề xuất:**

- **Concurrent runs cap (tùy chọn):** Nếu `RUNNER_MAX_CONCURRENT_RUNS > 0`, dùng `tokio::sync::Semaphore` (permits = giá trị đó). Khi có run mới, `spawn_run` acquire permit; khi run xong (success/fail) release. Tránh quá nhiều process/HTTP call đồng thời (CPU, memory, DB).
- **HTTP client:** Giữ timeout từ `adapterConfig.timeoutMs`; đảm bảo có upper bound (e.g. max 300_000 ms) để tránh config sai.
- **Process adapter:** Đã có timeout + grace; có thể đọc upper bound từ config (e.g. `RUNNER_PROCESS_MAX_TIMEOUT_SECS`) để không cho phép timeout quá lớn (bảo vệ tài nguyên).

**Rủi ro:** Trung bình. Semaphore cần test khi nhiều run cùng lúc (ví dụ scheduler + nhiều invoke).

---

### 2.5 Scheduler

**Hiện trạng:** `tokio::time::interval(60s)` cố định; mỗi tick gọi `run_heartbeat_scheduler_tick`.

**Đề xuất:**

- Đọc `SCHEDULER_INTERVAL_SECS` từ config (mặc định 60). Interval quá nhỏ (e.g. &lt; 10s) có thể tăng tải DB; nên document khuyến nghị (e.g. 30–120s).
- (Tùy chọn) Backoff khi tick lỗi: nếu `run_heartbeat_scheduler_tick` trả lỗi, lần sau chờ thêm vài giây rồi mới tick tiếp (tránh spam khi DB lỗi).
- (Tùy chọn) Jitter nhỏ trên interval để tránh nhiều instance cùng tick một lúc (quan trọng hơn khi chạy nhiều replica).

**Rủi ro:** Thấp. Chỉ đổi hằng số thành config và (nếu làm) thêm logic backoff đơn giản.

---

### 2.6 Build release

**Hiện trạng:** Có thể chưa tối ưu profile `release` (LTO, codegen units, strip).

**Đề xuất (trong `server-rs/Cargo.toml` hoặc `.cargo/config.toml`):**

- `[profile.release]`: `opt-level = 3`, `lto = true` (hoặc `"thin"` nếu build chậm), `codegen-units = 1` (tối ưu hơn, build chậm hơn).
- `strip = true` để bỏ symbol, giảm kích thước binary.
- Có thể thêm `panic = "abort"` để giảm kích thước (trade-off: không unwind).

**Rủi ro:** Thấp. Chỉ ảnh hưởng build time và kích thước/speed binary.

---

### 2.7 Observability

**Hiện trạng:** `tracing` + `tracing-subscriber` với `RUST_LOG`; log dạng text.

**Đề xuất (có thể làm từng bước):**

- **Request ID:** Middleware gắn `request_id` (UUID hoặc header `X-Request-Id` nếu client gửi) vào từng request; log mỗi request kèm `request_id` để trace.
- **Structured log (tùy chọn):** Output JSON (theo env, e.g. `LOG_FORMAT=json`) cho production để dễ đẩy vào hệ thống log (ELK, Loki, CloudWatch).
- **Metrics (sau này):** Endpoint `/api/health` mở rộng hoặc endpoint `/metrics` (Prometheus) với vài gauge/counter cơ bản (số request, số run đang chạy, lỗi,…). Có thể để phase sau khi đã ổn định config và HTTP.

**Rủi ro:** Trung bình (chủ yếu do thêm phụ thuộc và format log). Request ID ít rủi ro.

---

### 2.8 Bảo mật

**Hiện trạng:** CORS `Any`; chưa thấy security headers rõ ràng.

**Đề xuất:**

- **CORS:** Trong production (nhận diện qua env, e.g. `DEPLOYMENT_MODE=authenticated` hoặc `NODE_ENV=production`), không dùng `Any`; chỉ cho phép origin từ config (e.g. `CORS_ORIGINS=https://app.example.com`) hoặc list tách dấu phẩy.
- **Security headers:** Thêm middleware (tower hoặc axum) set headers: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY` (hoặc SAMEORIGIN nếu cần embed), `Referrer-Policy: strict-origin-when-cross-origin`. (CSP có thể phase sau vì dễ gãy nếu có inline script.)
- **Request size:** Đã nêu ở 2.3 (body limit); đủ để giảm surface DoS.

**Rủi ro:** Trung bình. CORS sai có thể chặn frontend; cần cấu hình đúng origin khi deploy.

---

## 3. Thứ tự triển khai đề xuất

1. **Phase 1 (cấu hình + DB + scheduler)**  
   - Mở rộng `Config` với tất cả env trong bảng 2.1.  
   - `db.rs` đọc `DB_POOL_*` từ config.  
   - `main.rs` đọc `SCHEDULER_INTERVAL_SECS`, truyền vào task scheduler.  
   - Cập nhật `doc/DEVELOPING.md` (và README nếu cần) liệt kê env.

2. **Phase 2 (HTTP + runner)**  
   - Body limit từ config; compression response.  
   - Runner: `RUNNER_MAX_CONCURRENT_RUNS` + semaphore; upper bound timeout HTTP/process.

3. **Phase 3 (build + observability + security)**  
   - Release profile (LTO, strip).  
   - Request ID middleware; (tùy chọn) structured log.  
   - CORS theo env; security headers.

4. **Phase 4 (tùy chọn)**  
   - Metrics endpoint.  
   - Config file (nếu cần).

---

## 4. Cập nhật tài liệu

- **doc/DEVELOPING.md:** Thêm mục "Environment variables (server-rs)" với bảng env và ý nghĩa.  
- **doc/DATABASE.md:** Ghi chú `DB_POOL_*` khi dùng Postgres.  
- **README.md:** Có thể thêm 1–2 dòng về env quan trọng (ví dụ `DATABASE_URL`, `PORT`).  
- **doc/RUST-MIGRATION-STATUS.md:** Sau khi làm xong, cập nhật ngắn "Backend đã áp dụng config và tối ưu theo RUST-BACKEND-OPTIMIZATION-PLAN.md".

---

## 5. Chấp nhận / Điều chỉnh

- Nếu bạn đồng ý với kế hoạch này, có thể bắt đầu từ Phase 1.  
- Nếu muốn bỏ bớt hoặc đổi thứ tự (ví dụ ưu tiên security trước), chỉ cần nêu rõ để điều chỉnh lại doc và thứ tự implement.  
- Nếu muốn thêm mục (ví dụ rate limit theo IP, health check sâu hơn), có thể bổ sung vào bảng 1 và mục 2 tương ứng.
