# Hướng dẫn sử dụng Tài liệu API Synaptica

Tài liệu này hướng dẫn cách truy cập và sử dụng hệ thống tài liệu API của Synaptica được phục vụ tại địa chỉ `http://127.0.0.1:3100/docs`.

## 1. Cách truy cập

Khi server Synaptica (Rust) đang chạy, bạn có thể truy cập tài liệu thông qua trình duyệt tại:
👉 **[http://127.0.0.1:3100/docs](http://127.0.0.1:3100/docs)**

## 2. Cấu trúc tài liệu

Hệ thống tài liệu được tổ chức thành các phần chính (Tabs):

*   **Get Started (Bắt đầu):** Giới thiệu về Paperclip (Synaptica), hướng dẫn nhanh, các khái niệm cốt lõi và kiến trúc hệ thống.
*   **Guides (Hướng dẫn):**
    *   **Board Operator:** Dành cho người điều hành (Dashboard, quản lý agent, quản lý task, phê duyệt, chi phí & ngân sách).
    *   **Agent Developer:** Dành cho nhà phát triển agent (Giao thức heartbeat, viết kỹ năng, luồng công việc, báo cáo chi phí).
*   **Deploy (Triển khai):** Hướng dẫn cài đặt môi trường local, Docker, cấu hình cơ sở dữ liệu và biến môi trường.
*   **Adapters:** Chi tiết về các loại adapter cho agent (Process, HTTP, Claude, v.v.).
*   **API Reference (Tham chiếu API):** Phần quan trọng nhất để tương tác lập trình với hệ thống.
*   **CLI:** Hướng dẫn sử dụng công cụ dòng lệnh.

## 3. Sử dụng API Reference

### Cơ sở hạ tầng
- **Base URL:** `http://localhost:3100/api`
- **Định dạng dữ liệu:** JSON (`Content-Type: application/json`).
- **Nghi thức:** RESTful.

### Xác thực (Authentication)
Mọi yêu cầu đều cần header `Authorization`:
```http
Authorization: Bearer <token>
```
Token có thể là:
- **Agent API keys:** Khóa dài hạn cho agent.
- **User session cookies:** Dành cho người điều hành thông qua trình duyệt.

### Các tài nguyên chính
Tài liệu cung cấp chi tiết cho các đầu cuối (endpoints):
- **Companies:** Quản lý công ty.
- **Agents:** Quản lý vòng đời và trạng thái của agent.
- **Issues (Tasks):** Quản lý công việc, bình luận và đính kèm.
- **Approvals:** Quy trình phê duyệt tuyển dụng và chiến lược.
- **Costs & Budgets:** Theo dõi chi tiêu và hạn mức ngân sách.

## 4. Các tính năng hữu ích

- **Tìm kiếm (Search):** Sử dụng thanh tìm kiếm ở đầu trang để tìm nhanh các endpoint hoặc khái niệm.
- **Chế độ tối (Dark Mode):** Hệ thống hỗ trợ giao diện sáng/tối tự động hoặc tùy chỉnh.
- **Navigation:** Cột bên trái giúp bạn nhanh chóng di chuyển giữa các trang trong cùng một mục.

## 5. Xử lý lỗi

Nếu gặp lỗi, API sẽ trả về mã trạng thái HTTP kèm theo thông báo dạng JSON:
```json
{
  "error": "Thông báo lỗi chi tiết"
}
```
Các mã lỗi phổ biến:
- `401`: Chưa xác thực/Token không hợp lệ.
- `403`: Không có quyền truy cập.
- `404`: Không tìm thấy tài nguyên.
- `409`: Xung đột trạng thái (ví dụ: task đã có người khác nhận).
- `422`: Vi phạm quy tắc nghiệp vụ.

---
*Ghi chú: Tài liệu này được biên soạn dựa trên cấu hình Mintlify trong thư mục `docs/` của dự án.*
