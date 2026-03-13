import type { CompanyPost } from "@paperclipai/shared";
import { api } from "./client";

export const companyPostsApi = {
  list: (companyId: string) =>
    api.get<CompanyPost[]>(`/companies/${companyId}/posts`),
  get: (companyId: string, postId: string) =>
    api.get<CompanyPost>(`/companies/${companyId}/posts/${postId}`),
  create: (
    companyId: string,
    data: { content: string; authorAgentId?: string | null; scheduledAt?: string | null }
  ) => api.post<CompanyPost>(`/companies/${companyId}/posts`, data),
  update: (
    companyId: string,
    postId: string,
    data: { content?: string; scheduledAt?: string | null }
  ) =>
    api.patch<CompanyPost>(`/companies/${companyId}/posts/${postId}`, data),
  remove: (companyId: string, postId: string) =>
    api.delete<void>(`/companies/${companyId}/posts/${postId}`),
};
