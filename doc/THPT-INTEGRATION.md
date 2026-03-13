# Tích hợp tính năng từ THPT Phước Bửu

Tài liệu mô tả nguồn tính năng và kế hoạch tích hợp từ dự án **thptphuocbuu** vào **Synaptica**.

## Nguồn

- **Repo:** `thptphuocbuu` (trong monorepo Synaptica, hoặc repo riêng)
- **Stack THPT:** Next.js 14 (App Router), React 18, Tailwind, PostgreSQL, Prisma, NextAuth
- **Mục đích:** Hệ thống LMS & mạng xã hội nội bộ trường THPT Phước Bửu

## Các tính năng đang tích hợp

### 1. Spaces (Không gian làm việc)

- **Trong THPT:** `Space` model (Prisma), có parent, members, departments. Trang `/dashboard/spaces`, API `/api/spaces`, tasks trong space.
- **Trong Synaptica:** Trang **Spaces** (`/spaces`) đã cập nhật mô tả và cấu trúc từ THPT. Tính năng: quản lý không gian làm việc, thành viên, liên kết tổ chuyên môn.
- **Cần thêm:** Bảng & API company-scoped (ví dụ `company_spaces`), CRUD, gắn với agents/projects nếu cần.

### 2. Tổ chuyên môn (Departments)

- **Trong THPT:** `Department` model, gắn space, leader (user), members, documents. Trang `/dashboard/departments`, API `/api/departments`.
- **Trong Synaptica:** Trang **Tổ chuyên môn** (`/departments`) đã cập nhật: tổ/bộ phận, tổ trưởng, tài liệu, space.
- **Cần thêm:** Bảng & API company-scoped (ví dụ `company_departments`), liên kết space, leader (agent hoặc user tùy mô hình).

### 3. Mạng xã hội (Social)

- **Trong THPT:** `Post` model, author, likes, comments, scheduled posts. Trang `/dashboard/social`, API `/api/posts`, `/api/posts/feed`, CreatePost + SocialFeed.
- **Trong Synaptica:** Trang **Mạng xã hội** (`/social`) đã cập nhật: bài đăng, bình luận, thích, kết bạn/theo dõi.
- **Cần thêm:** Bảng & API company-scoped (posts, post_likes, post_comments, follows?), feed và tạo bài viết.

### 4. Lớp học (Classes)

- **Trong THPT:** Lớp học, đăng ký môn, nộp bài, chấm bài, bảng thông báo. Trang `/dashboard/classes`, assignments.
- **Trong Synaptica:** Trang **Lớp học** (`/classes`) đã cập nhật: quản lý lớp, đăng ký môn, nộp/chấm bài, thông báo.
- **Cần thêm:** Bảng & API company-scoped (classes, class_members, assignments, submissions) nếu muốn LMS đầy đủ trong Synaptica.

### 5. DMS / Văn bản

- **Trong THPT:** Document workflow, incoming/outgoing, assign, approve. API `/api/dms/*`, `/api/documents/*`.
- **Trong Synaptica:** Đã có mục **Văn bản** (`/dms`) trong sidebar. Có thể tích hợp luồng công văn từ THPT sau.

## Nguyên tắc tích hợp

1. **Company-scoped:** Mọi entity mới (spaces, departments, posts, classes) đều gắn `company_id`, đồng bộ với mô hình multi-company của Synaptica.
2. **Contract đồng bộ:** Khi thêm schema/API, cập nhật `packages/db`, `packages/shared`, `server-rs`, `ui` theo đúng quy trình trong AGENTS.md.
3. **UI:** Giữ routing có prefix công ty (ví dụ `/THP/spaces`) và resolve company từ URL giống Boards/Projects.

## Trạng thái hiện tại

- **Đã làm:** Cập nhật UI bốn trang Spaces, Tổ chuyên môn, Mạng xã hội, Lớp học với nội dung và cấu trúc tính năng từ THPT; resolve company từ URL; thêm doc tích hợp.
- **Chưa làm:** Schema DB mới, API Rust, dữ liệu thật (CRUD, feed, v.v.) — sẽ bổ sung theo từng phase.

## Tham chiếu nhanh (THPT)

| Tính năng   | Trang THPT              | API chính THPT        |
|------------|-------------------------|------------------------|
| Spaces     | `app/dashboard/spaces/` | `app/api/spaces/`      |
| Departments| `app/dashboard/departments/` | (Prisma `department`) |
| Social     | `app/dashboard/social/` | `app/api/posts/`, feed |
| Classes    | `app/dashboard/classes/`| (classes, assignments) |
| DMS        | `app/dashboard/dms/`    | `app/api/dms/`         |
