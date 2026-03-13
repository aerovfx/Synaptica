# Kế hoạch tích hợp THPT – Theo dõi tiến độ

Bảng trạng thái theo phase và bước. Cập nhật trạng thái và ghi chú khi hoàn thành từng mục.

## Trạng thái theo phase

| Phase | Bước | Hạng mục | Trạng thái | Ghi chú |
|-------|------|----------|------------|---------|
| 1 | 1.1 | Schema company_spaces (packages/db) | Done | Migration 0029 |
| 1 | 1.2 | Shared type CompanySpace + API path | Done | packages/shared |
| 1 | 1.3 | API Rust spaces routes | Done | GET/POST/PATCH/DELETE /companies/:id/spaces |
| 1 | 1.4 | UI Spaces list + CRUD | Done | /spaces, list, tạo, xóa |
| 2 | 2.1 | Schema company_departments | Done | Migration 0030 |
| 2 | 2.2 | Shared + API Rust departments | Done | CRUD company-scoped |
| 2 | 2.3 | UI Departments list + CRUD, space dropdown | Done | /departments |
| 3 | 3.1 | Schema company_posts | Done | Migration 0031 |
| 3 | 3.2 | Shared + API Rust posts | Done | Feed, create, update, delete |
| 3 | 3.3 | UI Social feed + tạo bài | Done | /social |
| 4 | 4.1 | Schema company_classes | Done | Migration 0032 |
| 4 | 4.2 | Shared + API Rust classes | Done | CRUD /companies/:id/classes |
| 4 | 4.3 | UI Classes list + CRUD | Done | /classes |
| 5 | 5.1 | Schema company_dms_* (documents, incoming, outgoing) | Done | Migration 0033 |
| 5 | 5.2 | API Rust DMS GET list (documents, incoming, outgoing) | Done | /companies/:id/dms/* |
| 5 | 5.3 | UI DMS đã có sẵn, nối API | Done | /dms trả 200 + list |

## Cách chạy kiểm tra

**Kiểm tra kỹ thuật (theo AGENTS.md):**

```bash
pnpm -r typecheck
pnpm test:run
pnpm build
```

**Kiểm tra trên trình duyệt:** Mở `http://localhost:3100` (hoặc `http://127.0.0.1:3100/<prefix>/...` nếu dùng company prefix). Đảm bảo đã chạy `pnpm dev` và (nếu dùng DB) `pnpm db:migrate`.

| Phase | URL kiểm tra |
|-------|----------------|
| 1 – Spaces | `/<prefix>/spaces` – list, thêm space, xóa |
| 2 – Departments | `/<prefix>/departments` – list, thêm tổ, chọn space, xóa |
| 3 – Social | `/<prefix>/social` – feed, đăng bài, xóa bài |
| 4 – Classes | `/<prefix>/classes` – list lớp, thêm lớp, xóa |
| 5 – DMS (Văn bản) | `/<prefix>/dms` – tất cả / tài liệu chung / văn bản đến / văn bản đi |

**API (curl):**

```bash
# Health
curl http://localhost:3100/api/health

# Spaces (thay <companyId> bằng UUID công ty)
curl http://localhost:3100/api/companies/<companyId>/spaces

# Departments
curl http://localhost:3100/api/companies/<companyId>/departments

# Posts (feed)
curl http://localhost:3100/api/companies/<companyId>/posts

# Classes
curl http://localhost:3100/api/companies/<companyId>/classes

# DMS (văn bản)
curl http://localhost:3100/api/companies/<companyId>/dms/documents
curl http://localhost:3100/api/companies/<companyId>/dms/incoming
curl http://localhost:3100/api/companies/<companyId>/dms/outgoing
```
