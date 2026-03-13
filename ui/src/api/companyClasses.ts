import type { CompanyClass } from "@paperclipai/shared";
import { api } from "./client";

export const companyClassesApi = {
  list: (companyId: string) =>
    api.get<CompanyClass[]>(`/companies/${companyId}/classes`),
  get: (companyId: string, classId: string) =>
    api.get<CompanyClass>(`/companies/${companyId}/classes/${classId}`),
  create: (
    companyId: string,
    data: { name: string; description?: string | null; order?: number }
  ) => api.post<CompanyClass>(`/companies/${companyId}/classes`, data),
  update: (
    companyId: string,
    classId: string,
    data: { name?: string; description?: string | null; order?: number }
  ) =>
    api.patch<CompanyClass>(`/companies/${companyId}/classes/${classId}`, data),
  remove: (companyId: string, classId: string) =>
    api.delete<void>(`/companies/${companyId}/classes/${classId}`),
};
