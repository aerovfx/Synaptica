import type { Board, BoardColumn } from "@paperclipai/shared";
import { api } from "./client";

export const boardsApi = {
  list: (companyId: string) =>
    api.get<Board[]>(`/companies/${companyId}/boards`),
  get: (companyId: string, boardId: string) =>
    api.get<Board>(`/companies/${companyId}/boards/${boardId}`),
  create: (companyId: string, data: { name: string; projectId?: string; type?: string }) =>
    api.post<Board>(`/companies/${companyId}/boards`, data),
  update: (companyId: string, boardId: string, data: Record<string, unknown>) =>
    api.patch<Board>(`/companies/${companyId}/boards/${boardId}`, data),
  remove: (companyId: string, boardId: string) =>
    api.delete<void>(`/companies/${companyId}/boards/${boardId}`),

  listColumns: (companyId: string, boardId: string) =>
    api.get<BoardColumn[]>(`/companies/${companyId}/boards/${boardId}/columns`),
  createColumn: (companyId: string, boardId: string, data: { name: string; position?: number }) =>
    api.post<BoardColumn>(`/companies/${companyId}/boards/${boardId}/columns`, data),
  updateColumn: (
    companyId: string,
    boardId: string,
    columnId: string,
    data: { name?: string; position?: number }
  ) =>
    api.patch<BoardColumn>(
      `/companies/${companyId}/boards/${boardId}/columns/${columnId}`,
      data
    ),
  removeColumn: (companyId: string, boardId: string, columnId: string) =>
    api.delete<void>(`/companies/${companyId}/boards/${boardId}/columns/${columnId}`),
};
