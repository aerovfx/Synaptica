import type { Project, ProjectWorkspace } from "@paperclipai/shared";
import { deriveProjectUrlKey, deriveProjectUrlKeyLegacy, isUuidLike } from "@paperclipai/shared";
import { api } from "./client";
import { ApiError } from "./client";

function withCompanyScope(path: string, companyId?: string) {
  if (!companyId) return path;
  const separator = path.includes("?") ? "&" : "?";
  return `${path}${separator}companyId=${encodeURIComponent(companyId)}`;
}

function projectPath(id: string, companyId?: string, suffix = "") {
  return withCompanyScope(`/projects/${encodeURIComponent(id)}${suffix}`, companyId);
}

/** Resolve project by slug (canonical or legacy). Used when URL has slug instead of UUID. */
async function getProjectBySlug(slug: string, companyId: string): Promise<Project | null> {
  const list = await api.get<Project[]>(`/companies/${companyId}/projects`);
  const slugLower = slug.toLowerCase();
  return (
    list.find(
      (p) =>
        deriveProjectUrlKey(p.name, p.id) === slugLower ||
        deriveProjectUrlKeyLegacy(p.name, p.id) === slugLower,
    ) ?? null
  );
}

export const projectsApi = {
  list: (companyId: string) => api.get<Project[]>(`/companies/${companyId}/projects`),
  get: async (id: string, companyId?: string): Promise<Project> => {
    // Backend only accepts UUID; non-UUID (slug like "d" or "du-l-ch") causes 500. Never call backend for slug.
    if (!isUuidLike(id)) {
      if (!companyId) {
        throw new ApiError("Company required to resolve project by slug", 400, null);
      }
      const bySlug = await getProjectBySlug(id, companyId);
      if (bySlug) return bySlug;
      throw new ApiError("Project not found", 404, null);
    }
    try {
      return await api.get<Project>(projectPath(id, companyId));
    } catch (e) {
      if (e instanceof ApiError && e.status === 404 && companyId) {
        const bySlug = await getProjectBySlug(id, companyId);
        if (bySlug) return bySlug;
      }
      throw e;
    }
  },
  create: (companyId: string, data: Record<string, unknown>) =>
    api.post<Project>(`/companies/${companyId}/projects`, data),
  update: (id: string, data: Record<string, unknown>, companyId?: string) =>
    api.patch<Project>(projectPath(id, companyId), data),
  listWorkspaces: (projectId: string, companyId?: string) =>
    api.get<ProjectWorkspace[]>(projectPath(projectId, companyId, "/workspaces")),
  createWorkspace: (projectId: string, data: Record<string, unknown>, companyId?: string) =>
    api.post<ProjectWorkspace>(projectPath(projectId, companyId, "/workspaces"), data),
  updateWorkspace: (projectId: string, workspaceId: string, data: Record<string, unknown>, companyId?: string) =>
    api.patch<ProjectWorkspace>(
      projectPath(projectId, companyId, `/workspaces/${encodeURIComponent(workspaceId)}`),
      data,
    ),
  removeWorkspace: (projectId: string, workspaceId: string, companyId?: string) =>
    api.delete<ProjectWorkspace>(projectPath(projectId, companyId, `/workspaces/${encodeURIComponent(workspaceId)}`)),
  remove: (id: string, companyId?: string) => api.delete<Project>(projectPath(id, companyId)),
};
