import { useState, useEffect, useMemo } from "react";
import { useParams } from "@/lib/router";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useCompany } from "../context/CompanyContext";
import { useBreadcrumbs } from "../context/BreadcrumbContext";
import { queryKeys } from "../lib/queryKeys";
import { companyClassesApi } from "../api/companyClasses";
import { PageSkeleton } from "../components/PageSkeleton";
import { EmptyState } from "../components/EmptyState";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { BookOpen, Plus, Trash2 } from "lucide-react";
import type { CompanyClass } from "@paperclipai/shared";

export function Classes() {
  const params = useParams<{ companyPrefix?: string }>();
  const { selectedCompanyId, companies = [], loading: companiesLoading } = useCompany();
  const { setBreadcrumbs } = useBreadcrumbs();
  const queryClient = useQueryClient();
  const [showForm, setShowForm] = useState(false);
  const [newName, setNewName] = useState("");
  const [newDescription, setNewDescription] = useState("");

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
    setBreadcrumbs([{ label: "Lớp học" }]);
  }, [setBreadcrumbs]);

  const { data: classes = [], isLoading, error } = useQuery({
    queryKey: queryKeys.companyClasses.list(lookupCompanyId!),
    queryFn: () => companyClassesApi.list(lookupCompanyId!),
    enabled: !!lookupCompanyId,
  });

  const createClass = useMutation({
    mutationFn: (data: { name: string; description?: string | null; order?: number }) =>
      companyClassesApi.create(lookupCompanyId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.companyClasses.list(lookupCompanyId!),
      });
      setNewName("");
      setNewDescription("");
      setShowForm(false);
    },
  });

  const deleteClass = useMutation({
    mutationFn: (classId: string) => companyClassesApi.remove(lookupCompanyId!, classId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.companyClasses.list(lookupCompanyId!),
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
          Company not found for prefix &quot;{companyPrefix}&quot;. Check the URL or select a company
          from the sidebar.
        </p>
      </div>
    );
  }

  if (!lookupCompanyId) {
    if (companyPrefix || companiesLoading) return <PageSkeleton variant="list" />;
    return (
      <EmptyState icon={BookOpen} message="Select a company to view lớp học." />
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
            <BookOpen className="h-6 w-6 text-muted-foreground" />
            Lớp học
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Quản lý lớp học (tích hợp từ THPT Phước Bửu).
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setShowForm((v) => !v)}
          className="shrink-0"
        >
          <Plus className="h-4 w-4 mr-1" />
          Thêm lớp
        </Button>
      </div>

      {showForm && (
        <div className="rounded-lg border border-border bg-card p-4 space-y-3">
          <h3 className="text-sm font-medium">Tạo lớp mới</h3>
          <div className="grid gap-2 sm:grid-cols-2">
            <Input
              placeholder="Tên lớp"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
            />
            <Input
              placeholder="Mô tả (tùy chọn)"
              value={newDescription}
              onChange={(e) => setNewDescription(e.target.value)}
            />
          </div>
          <div className="flex gap-2">
            <Button
              size="sm"
              onClick={() =>
                createClass.mutate({
                  name: newName.trim(),
                  description: newDescription.trim() || null,
                })
              }
              disabled={!newName.trim() || createClass.isPending}
            >
              Tạo
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                setShowForm(false);
                setNewName("");
                setNewDescription("");
              }}
            >
              Hủy
            </Button>
          </div>
          {createClass.isError && (
            <p className="text-sm text-destructive">
              {createClass.error instanceof Error ? createClass.error.message : "Lỗi khi tạo lớp."}
            </p>
          )}
        </div>
      )}

      {isLoading ? (
        <PageSkeleton variant="list" />
      ) : error ? (
        <p className="text-sm text-destructive">
          {error instanceof Error ? error.message : "Không tải được danh sách lớp."}
        </p>
      ) : classes.length === 0 ? (
        <div className="rounded-lg border border-dashed border-border bg-muted/20 p-6 text-center text-sm text-muted-foreground">
          Chưa có lớp nào. Bấm &quot;Thêm lớp&quot; để tạo.
        </div>
      ) : (
        <ul className="grid gap-2 list-none p-0 m-0">
          {classes.map((c: CompanyClass) => (
            <li
              key={c.id}
              className="rounded-lg border border-border bg-card p-3 flex items-center justify-between gap-2"
            >
              <div className="min-w-0">
                <span className="font-medium block truncate">{c.name}</span>
                {c.description && (
                  <span className="text-xs text-muted-foreground line-clamp-1">{c.description}</span>
                )}
              </div>
              <Button
                variant="ghost"
                size="icon"
                className="shrink-0 text-muted-foreground hover:text-destructive"
                onClick={() => deleteClass.mutate(c.id)}
                disabled={deleteClass.isPending}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
