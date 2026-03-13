import type { CompanySpace } from "@paperclipai/shared";
import { api } from "./client";

export const companySpacesApi = {
  list: (companyId: string) =>
    api.get<CompanySpace[]>(`/companies/${companyId}/spaces`),
  get: (companyId: string, spaceId: string) =>
    api.get<CompanySpace>(`/companies/${companyId}/spaces/${spaceId}`),
  create: (
    companyId: string,
    data: { name: string; parentId?: string | null; order?: number }
  ) => api.post<CompanySpace>(`/companies/${companyId}/spaces`, data),
  update: (
    companyId: string,
    spaceId: string,
    data: { name?: string; parentId?: string | null; order?: number }
  ) =>
    api.patch<CompanySpace>(`/companies/${companyId}/spaces/${spaceId}`, data),
  remove: (companyId: string, spaceId: string) =>
    api.delete<void>(`/companies/${companyId}/spaces/${spaceId}`),
};
