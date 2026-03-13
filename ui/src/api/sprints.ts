import type { Sprint } from "@paperclipai/shared";
import { api } from "./client";

export const sprintsApi = {
  list: (companyId: string, boardId: string) =>
    api.get<Sprint[]>(`/companies/${companyId}/boards/${boardId}/sprints`),
  get: (companyId: string, boardId: string, sprintId: string) =>
    api.get<Sprint>(`/companies/${companyId}/boards/${boardId}/sprints/${sprintId}`),
  create: (
    companyId: string,
    boardId: string,
    data: { name: string; startDate?: string; endDate?: string; status?: string }
  ) =>
    api.post<Sprint>(`/companies/${companyId}/boards/${boardId}/sprints`, data),
  update: (
    companyId: string,
    boardId: string,
    sprintId: string,
    data: Record<string, unknown>
  ) =>
    api.patch<Sprint>(
      `/companies/${companyId}/boards/${boardId}/sprints/${sprintId}`,
      data
    ),
  remove: (companyId: string, boardId: string, sprintId: string) =>
    api.delete<void>(`/companies/${companyId}/boards/${boardId}/sprints/${sprintId}`),
};
