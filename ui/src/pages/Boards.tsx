import { useState, useEffect, useMemo } from "react";
import { useParams } from "@/lib/router";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Link } from "@/lib/router";
import { boardsApi } from "../api/boards";
import { projectsApi } from "../api/projects";
import { useCompany } from "../context/CompanyContext";
import { useBreadcrumbs } from "../context/BreadcrumbContext";
import { queryKeys } from "../lib/queryKeys";
import { EmptyState } from "../components/EmptyState";
import { PageSkeleton } from "../components/PageSkeleton";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { LayoutGrid, Plus, ChevronRight } from "lucide-react";
import type { Board } from "@paperclipai/shared";

export function Boards() {
  const params = useParams<{ companyPrefix?: string }>();
  const { selectedCompanyId, companies = [], loading: companiesLoading } = useCompany();
  const { setBreadcrumbs } = useBreadcrumbs();
  const queryClient = useQueryClient();
  const [showForm, setShowForm] = useState(false);
  const [newName, setNewName] = useState("");
  const [newProjectId, setNewProjectId] = useState("");

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
    setBreadcrumbs([{ label: "Boards" }]);
  }, [setBreadcrumbs]);

  const { data: boards, isLoading, error } = useQuery({
    queryKey: queryKeys.boards.list(lookupCompanyId!),
    queryFn: () => boardsApi.list(lookupCompanyId!),
    enabled: !!lookupCompanyId,
  });

  const { data: projects } = useQuery({
    queryKey: queryKeys.projects.list(lookupCompanyId!),
    queryFn: () => projectsApi.list(lookupCompanyId!),
    enabled: !!lookupCompanyId && showForm,
  });

  const createBoard = useMutation({
    mutationFn: (data: { name: string; projectId?: string }) =>
      boardsApi.create(lookupCompanyId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.boards.list(lookupCompanyId!) });
      setNewName("");
      setNewProjectId("");
      setShowForm(false);
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
    return <EmptyState icon={LayoutGrid} message="Select a company to view boards." />;
  }

  if (isLoading) {
    return <PageSkeleton variant="list" />;
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">Boards</h1>
          <p className="text-sm text-muted-foreground mt-0.5">
            Kanban boards for issues. Open a board to manage columns and cards.
          </p>
        </div>
        <div className="flex items-center gap-2 flex-wrap shrink-0">
        {showForm ? (
          <div className="flex items-center gap-2 flex-wrap">
            <Input
              type="text"
              placeholder="Board name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              className="h-9 w-full sm:w-[200px]"
            />
            <select
              value={newProjectId}
              onChange={(e) => setNewProjectId(e.target.value)}
              className="h-9 rounded-md border border-input bg-background px-3 text-sm min-w-[160px]"
            >
              <option value="">No project</option>
              {(projects ?? []).map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
            <Button
              size="sm"
              onClick={() => {
                if (newName.trim()) {
                  createBoard.mutate({
                    name: newName.trim(),
                    projectId: newProjectId || undefined,
                  });
                }
              }}
              disabled={!newName.trim() || createBoard.isPending}
            >
              Create
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setShowForm(false)}>
              Cancel
            </Button>
          </div>
        ) : (
          <Button size="sm" onClick={() => setShowForm(true)}>
            <Plus className="h-4 w-4 mr-1.5" />
            Add board
          </Button>
        )}
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error.message}</p>}
      {createBoard.error && (
        <p className="text-sm text-destructive">{createBoard.error.message}</p>
      )}

      {boards && boards.length === 0 && !showForm && (
        <EmptyState
          icon={LayoutGrid}
          message="No boards yet. Create a board to manage issues in columns (Kanban)."
          action="Add board"
          onAction={() => setShowForm(true)}
        />
      )}

      {boards && boards.length > 0 && (
        <ul className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 list-none p-0 m-0">
          {boards.map((board: Board) => (
            <li key={board.id}>
              <Link
                to={`/boards/${board.id}`}
                className="flex items-center gap-3 px-4 py-3 rounded-lg border border-border bg-card hover:bg-accent/20 hover:border-accent/50 transition-colors"
              >
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted/50">
                  <LayoutGrid className="h-5 w-5 text-muted-foreground" />
                </div>
                <div className="flex-1 min-w-0">
                  <span className="font-medium truncate block">{board.name}</span>
                  <span className="text-xs text-muted-foreground">
                    {board.type ?? "kanban"}
                    {board.projectId ? " · Project" : ""}
                  </span>
                </div>
                <ChevronRight className="h-4 w-4 text-muted-foreground shrink-0" />
              </Link>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
