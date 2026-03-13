import type { CompanyDepartment } from "@paperclipai/shared";
import { api } from "./client";

export const companyDepartmentsApi = {
  list: (companyId: string) =>
    api.get<CompanyDepartment[]>(`/companies/${companyId}/departments`),
  get: (companyId: string, departmentId: string) =>
    api.get<CompanyDepartment>(`/companies/${companyId}/departments/${departmentId}`),
  create: (
    companyId: string,
    data: { name: string; spaceId?: string | null; leaderAgentId?: string | null; order?: number }
  ) => api.post<CompanyDepartment>(`/companies/${companyId}/departments`, data),
  update: (
    companyId: string,
    departmentId: string,
    data: {
      name?: string;
      spaceId?: string | null;
      leaderAgentId?: string | null;
      order?: number;
    }
  ) =>
    api.patch<CompanyDepartment>(
      `/companies/${companyId}/departments/${departmentId}`,
      data
    ),
  remove: (companyId: string, departmentId: string) =>
    api.delete<void>(`/companies/${companyId}/departments/${departmentId}`),
};
