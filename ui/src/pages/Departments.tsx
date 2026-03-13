import { useState, useEffect, useMemo } from "react";
import { useParams } from "@/lib/router";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useCompany } from "../context/CompanyContext";
import { useBreadcrumbs } from "../context/BreadcrumbContext";
import { queryKeys } from "../lib/queryKeys";
import { companyDepartmentsApi } from "../api/companyDepartments";
import { companySpacesApi } from "../api/companySpaces";
import { PageSkeleton } from "../components/PageSkeleton";
import { EmptyState } from "../components/EmptyState";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Briefcase, User, FileText, Building2, Plus, Trash2 } from "lucide-react";
import type { CompanyDepartment } from "@paperclipai/shared";

export function Departments() {
  const params = useParams<{ companyPrefix?: string }>();
  const { selectedCompanyId, companies = [], loading: companiesLoading } = useCompany();
  const { setBreadcrumbs } = useBreadcrumbs();
  const queryClient = useQueryClient();
  const [showForm, setShowForm] = useState(false);
  const [newName, setNewName] = useState("");
  const [newSpaceId, setNewSpaceId] = useState<string>("");

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
    setBreadcrumbs([{ label: "Tổ chuyên môn" }]);
  }, [setBreadcrumbs]);

  const { data: departments = [], isLoading, error } = useQuery({
    queryKey: queryKeys.companyDepartments.list(lookupCompanyId!),
    queryFn: () => companyDepartmentsApi.list(lookupCompanyId!),
    enabled: !!lookupCompanyId,
  });

  const { data: spaces = [] } = useQuery({
    queryKey: queryKeys.companySpaces.list(lookupCompanyId!),
    queryFn: () => companySpacesApi.list(lookupCompanyId!),
    enabled: !!lookupCompanyId && showForm,
  });

  const createDepartment = useMutation({
    mutationFn: (data: {
      name: string;
      spaceId?: string | null;
      order?: number;
    }) => companyDepartmentsApi.create(lookupCompanyId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.companyDepartments.list(lookupCompanyId!),
      });
      setNewName("");
      setNewSpaceId("");
      setShowForm(false);
    },
  });

  const deleteDepartment = useMutation({
    mutationFn: (departmentId: string) =>
      companyDepartmentsApi.remove(lookupCompanyId!, departmentId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.companyDepartments.list(lookupCompanyId!),
      });
    },
  });

  const hasUnknownPrefix =
    Boolean(companyPrefix) &&
    !companiesLoading &&
    companies.length > 0 &&
    !routeCompanyId;

  if (hasUnknownPrefix) {
    return (
      <div className="py-8 text-center">
        <p className="text-sm text-destructive">
          Company not found for prefix &quot;{companyPrefix}&quot;. Check the URL or select a
          company from the sidebar.
        </p>
      </div>
    );
  }

  if (!lookupCompanyId) {
    if (companyPrefix || companiesLoading) return <PageSkeleton variant="list" />;
    return (
      <EmptyState
        icon={Briefcase}
        message="Select a company to view tổ chuyên môn."
      />
    );
  }

  if (isLoading) return <PageSkeleton variant="list" />;

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
            <Briefcase className="h-6 w-6 text-muted-foreground" />
            Tổ chuyên môn
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Quản lý các tổ chuyên môn và bộ phận trong trường (tích hợp từ THPT Phước Bửu).
          </p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          {showForm ? (
            <div className="flex items-center gap-2 flex-wrap">
              <Input
                type="text"
                placeholder="Tên tổ"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                className="h-9 w-full sm:w-[200px]"
              />
              <select
                value={newSpaceId}
                onChange={(e) => setNewSpaceId(e.target.value)}
                className="h-9 rounded-md border border-input bg-background px-3 text-sm min-w-[140px]"
              >
                <option value="">Không gắn space</option>
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
                    createDepartment.mutate({
                      name: newName.trim(),
                      spaceId: newSpaceId || undefined,
                    });
                  }
                }}
                disabled={!newName.trim() || createDepartment.isPending}
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
              Thêm tổ
            </Button>
          )}
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error.message}</p>}
      {createDepartment.error && (
        <p className="text-sm text-destructive">{createDepartment.error.message}</p>
      )}

      {departments.length === 0 && !showForm && (
        <EmptyState
          icon={Briefcase}
          message="Chưa có tổ chuyên môn nào. Tạo tổ để quản lý bộ phận."
          action="Thêm tổ"
          onAction={() => setShowForm(true)}
        />
      )}

      {departments.length > 0 && (
        <ul className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 list-none p-0 m-0">
          {departments.map((dept: CompanyDepartment) => (
            <li
              key={dept.id}
              className="flex items-center gap-3 px-4 py-3 rounded-lg border border-border bg-card hover:bg-accent/10 transition-colors"
            >
              <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted/50">
                <Briefcase className="h-5 w-5 text-muted-foreground" />
              </div>
              <div className="flex-1 min-w-0">
                <span className="font-medium truncate block">{dept.name}</span>
                <span className="text-xs text-muted-foreground">
                  {dept.spaceId ? "Đã gắn space" : "Chưa gắn space"} · Thứ tự {dept.order}
                </span>
              </div>
              <Button
                variant="ghost"
                size="icon-sm"
                className="h-8 w-8 shrink-0"
                onClick={() => deleteDepartment.mutate(dept.id)}
                disabled={deleteDepartment.isPending}
                title="Xóa"
              >
                <Trash2 className="h-4 w-4 text-muted-foreground" />
              </Button>
            </li>
          ))}
        </ul>
      )}

      <ul className="grid gap-3 sm:grid-cols-2 list-none p-0 m-0 pt-4 border-t border-border">
        <li className="rounded-lg border border-border bg-card p-4 flex items-start gap-3">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted/50">
            <User className="h-5 w-5 text-muted-foreground" />
          </div>
          <div>
            <span className="font-medium block">Tổ trưởng</span>
            <span className="text-xs text-muted-foreground">
              Chỉ định tổ trưởng (agent) cho tổ (sắp có).
            </span>
          </div>
        </li>
        <li className="rounded-lg border border-border bg-card p-4 flex items-start gap-3">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted/50">
            <FileText className="h-5 w-5 text-muted-foreground" />
          </div>
          <div>
            <span className="font-medium block">Tài liệu</span>
            <span className="text-xs text-muted-foreground">
              Tài liệu nội bộ tổ (sắp có).
            </span>
          </div>
        </li>
      </ul>
    </div>
  );
}
