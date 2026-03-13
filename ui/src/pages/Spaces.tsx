import { useState, useEffect, useMemo } from "react";
import { useParams } from "@/lib/router";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useCompany } from "../context/CompanyContext";
import { useBreadcrumbs } from "../context/BreadcrumbContext";
import { queryKeys } from "../lib/queryKeys";
import { companySpacesApi } from "../api/companySpaces";
import { PageSkeleton } from "../components/PageSkeleton";
import { EmptyState } from "../components/EmptyState";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Building2, Users, Briefcase, Plus, ChevronRight, Trash2 } from "lucide-react";
import type { CompanySpace } from "@paperclipai/shared";

export function Spaces() {
  const params = useParams<{ companyPrefix?: string }>();
  const { selectedCompanyId, companies = [], loading: companiesLoading } = useCompany();
  const { setBreadcrumbs } = useBreadcrumbs();
  const queryClient = useQueryClient();
  const [showForm, setShowForm] = useState(false);
  const [newName, setNewName] = useState("");
  const [newParentId, setNewParentId] = useState<string>("");

  const companyPrefix = params.companyPrefix?.trim();
  const routeCompanyId = useMemo(() => {
    if (!companyPrefix) return null;
    const requested = companyPrefix.toUpperCase();
    return (
      companies.find((c) => (c.issuePrefix ?? "").trim().toUpperCase() === requested)?.id ?? null
    );
  }, [companies, companyPrefix]);
  const lookupCompanyId = routeCompanyId ?? selectedCompanyId;

  useEffect(() => {
    setBreadcrumbs([{ label: "Spaces" }]);
  }, [setBreadcrumbs]);

  const { data: spaces = [], isLoading, error } = useQuery({
    queryKey: queryKeys.companySpaces.list(lookupCompanyId!),
    queryFn: () => companySpacesApi.list(lookupCompanyId!),
    enabled: !!lookupCompanyId,
  });

  const createSpace = useMutation({
    mutationFn: (data: { name: string; parentId?: string | null; order?: number }) =>
      companySpacesApi.create(lookupCompanyId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.companySpaces.list(lookupCompanyId!) });
      setNewName("");
      setNewParentId("");
      setShowForm(false);
    },
  });

  const deleteSpace = useMutation({
    mutationFn: (spaceId: string) => companySpacesApi.remove(lookupCompanyId!, spaceId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.companySpaces.list(lookupCompanyId!) });
    },
  });

  const hasUnknownPrefix = Boolean(companyPrefix) && !companiesLoading && companies.length > 0 && !routeCompanyId;

  if (hasUnknownPrefix) {
    return (
      <div className="py-8 text-center">
        <p className="text-sm text-destructive">
          Company not found for prefix &quot;{companyPrefix}&quot;. Check the URL or select a company from the sidebar.
        </p>
      </div>
    );
  }

  if (!lookupCompanyId) {
    if (companyPrefix || companiesLoading) return <PageSkeleton variant="list" />;
    return <EmptyState icon={Building2} message="Select a company to view spaces." />;
  }

  if (isLoading) return <PageSkeleton variant="list" />;

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
            <Building2 className="h-6 w-6 text-muted-foreground" />
            Không gian làm việc
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Quản lý các không gian làm việc và tổ chức trong trường (tích hợp từ THPT Phước Bửu).
          </p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          {showForm ? (
            <div className="flex items-center gap-2 flex-wrap">
              <Input
                type="text"
                placeholder="Tên space"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                className="h-9 w-full sm:w-[200px]"
              />
              <select
                value={newParentId}
                onChange={(e) => setNewParentId(e.target.value)}
                className="h-9 rounded-md border border-input bg-background px-3 text-sm min-w-[140px]"
              >
                <option value="">Không có space cha</option>
                {spaces.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.name}
                  </option>
                ))}
              </select>
              <Button
                size="sm"
                onClick={() => {
                  if (newName.trim()) {
                    createSpace.mutate({
                      name: newName.trim(),
                      parentId: newParentId || undefined,
                    });
                  }
                }}
                disabled={!newName.trim() || createSpace.isPending}
              >
                Tạo
              </Button>
              <Button size="sm" variant="ghost" onClick={() => setShowForm(false)}>
                Hủy
              </Button>
            </div>
          ) : (
            <Button size="sm" onClick={() => setShowForm(true)}>
              <Plus className="h-4 w-4 mr-1.5" />
              Thêm space
            </Button>
          )}
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error.message}</p>}
      {createSpace.error && (
        <p className="text-sm text-destructive">{createSpace.error.message}</p>
      )}

      {spaces.length === 0 && !showForm && (
        <EmptyState
          icon={Building2}
          message="Chưa có space nào. Tạo space để quản lý không gian làm việc."
          action="Thêm space"
          onAction={() => setShowForm(true)}
        />
      )}

      {spaces.length > 0 && (
        <ul className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 list-none p-0 m-0">
          {spaces.map((space: CompanySpace) => (
            <li
              key={space.id}
              className="flex items-center gap-3 px-4 py-3 rounded-lg border border-border bg-card hover:bg-accent/10 transition-colors"
            >
              <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted/50">
                <Building2 className="h-5 w-5 text-muted-foreground" />
              </div>
              <div className="flex-1 min-w-0">
                <span className="font-medium truncate block">{space.name}</span>
                <span className="text-xs text-muted-foreground">
                  {space.parentId ? "Space con" : "Space gốc"} · Thứ tự {space.order}
                </span>
              </div>
              <div className="flex items-center gap-1 shrink-0">
                <Button
                  variant="ghost"
                  size="icon-sm"
                  className="h-8 w-8"
                  onClick={() => deleteSpace.mutate(space.id)}
                  disabled={deleteSpace.isPending}
                  title="Xóa"
                >
                  <Trash2 className="h-4 w-4 text-muted-foreground" />
                </Button>
                <ChevronRight className="h-4 w-4 text-muted-foreground" />
              </div>
            </li>
          ))}
        </ul>
      )}

      <ul className="grid gap-3 sm:grid-cols-2 list-none p-0 m-0 pt-4 border-t border-border">
        <li className="rounded-lg border border-border bg-card p-4 flex items-start gap-3">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted/50">
            <Users className="h-5 w-5 text-muted-foreground" />
          </div>
          <div>
            <span className="font-medium block">Thành viên</span>
            <span className="text-xs text-muted-foreground">
              Mời thành viên vào space, phân quyền truy cập (sắp có).
            </span>
          </div>
        </li>
        <li className="rounded-lg border border-border bg-card p-4 flex items-start gap-3">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted/50">
            <Briefcase className="h-5 w-5 text-muted-foreground" />
          </div>
          <div>
            <span className="font-medium block">Liên kết tổ chuyên môn</span>
            <span className="text-xs text-muted-foreground">
              Mỗi space có thể gắn với một hoặc nhiều tổ chuyên môn (sắp có).
            </span>
          </div>
        </li>
      </ul>
    </div>
  );
}
