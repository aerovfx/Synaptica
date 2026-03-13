import { useMemo, useState, useEffect, useCallback } from "react";
import { useParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Link } from "@/lib/router";
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  useDroppable,
  type DragStartEvent,
  type DragEndEvent,
} from "@dnd-kit/core";
import { SortableContext, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { boardsApi } from "../api/boards";
import { issuesApi } from "../api/issues";
import { agentsApi } from "../api/agents";
import { heartbeatsApi } from "../api/heartbeats";
import { useCompany } from "../context/CompanyContext";
import { useBreadcrumbs } from "../context/BreadcrumbContext";
import { queryKeys } from "../lib/queryKeys";
import { EmptyState } from "../components/EmptyState";
import { PageSkeleton } from "../components/PageSkeleton";
import { PriorityIcon } from "../components/PriorityIcon";
import { Identity } from "../components/Identity";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { LayoutGrid, Plus, MoreHorizontal, Share2, Star, ChevronDown, Inbox, Calendar } from "lucide-react";
import type { BoardColumn as BoardColumnType, Issue } from "@paperclipai/shared";

const UNPLACED = "__unplaced__";

interface Agent {
  id: string;
  name: string;
}

function deriveInitials(name: string): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length >= 2) return (parts[0][0]! + parts[parts.length - 1]![0]).toUpperCase();
  return name.slice(0, 2).toUpperCase();
}

function BoardHeader({
  boardName,
  assigneeIds,
  agents,
  onShare,
}: {
  boardName: string;
  assigneeIds: string[];
  agents?: Agent[];
  onShare?: () => void;
}) {
  const uniqueIds = useMemo(() => Array.from(new Set(assigneeIds)), [assigneeIds]);
  const displayCount = 5;
  const showIds = uniqueIds.slice(0, displayCount);
  const extra = uniqueIds.length > displayCount ? uniqueIds.length - displayCount : 0;
  const agentName = (id: string) => agents?.find((a) => a.id === id)?.name ?? id.slice(0, 8);

  return (
    <header className="sticky top-0 z-10 flex items-center justify-between gap-4 h-12 px-4 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80">
      <div className="flex items-center gap-2 min-w-0">
        <h1 className="text-lg font-bold truncate">{boardName}</h1>
        <ChevronDown className="h-4 w-4 text-muted-foreground shrink-0" aria-hidden />
      </div>
      <div className="flex items-center gap-2 shrink-0">
        <div className="flex -space-x-2">
          {showIds.map((id) => (
            <Avatar key={id} size="xs" className="ring-2 ring-background rounded-full">
              <AvatarFallback className="text-[10px] bg-muted text-muted-foreground">
                {deriveInitials(agentName(id))}
              </AvatarFallback>
            </Avatar>
          ))}
          {extra > 0 && (
            <span className="inline-flex h-5 w-5 items-center justify-center rounded-full bg-muted text-[10px] font-medium text-muted-foreground ring-2 ring-background">
              +{extra}
            </span>
          )}
        </div>
        <Button variant="ghost" size="icon-sm" onClick={onShare} title="Share">
          <Share2 className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon-sm" title="Favorite">
          <Star className="h-4 w-4" />
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon-sm" title="More">
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem asChild>
              <Link to="/boards">Switch boards</Link>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  );
}

function KanbanColumnCard({
  issue,
  agents,
  isLive,
  isOverlay,
}: {
  issue: Issue;
  agents?: Agent[];
  isLive?: boolean;
  isOverlay?: boolean;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: issue.id, data: { issue } });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  const agentName = (id: string | null) => {
    if (!id || !agents) return null;
    return agents.find((a) => a.id === id)?.name ?? null;
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      className={`rounded-md border bg-card p-2.5 cursor-grab active:cursor-grabbing transition-shadow ${
        isDragging && !isOverlay ? "opacity-30" : ""
      } ${isOverlay ? "shadow-lg ring-1 ring-primary/20" : "hover:shadow-sm"}`}
    >
      <Link
        to={`/issues/${issue.identifier ?? issue.id}`}
        className="block no-underline text-inherit"
        onClick={(e) => {
          if ((e as React.MouseEvent & { _dragging?: boolean })._dragging) e.preventDefault();
        }}
      >
        <div className="flex items-start gap-1.5 mb-1.5">
          <span className="text-xs text-muted-foreground font-mono shrink-0">
            {issue.identifier ?? issue.id.slice(0, 8)}
          </span>
          {isLive && (
            <span className="relative flex h-2 w-2 shrink-0 mt-0.5">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75" />
              <span className="relative inline-flex rounded-full h-2 w-2 bg-blue-500" />
            </span>
          )}
        </div>
        <p className="text-sm leading-snug line-clamp-2 mb-2">{issue.title}</p>
        <div className="flex items-center gap-2">
          <PriorityIcon priority={issue.priority} />
          {issue.assigneeAgentId &&
            (() => {
              const name = agentName(issue.assigneeAgentId);
              return name ? (
                <Identity name={name} size="xs" />
              ) : (
                <span className="text-xs text-muted-foreground font-mono">
                  {issue.assigneeAgentId.slice(0, 8)}
                </span>
              );
            })()}
        </div>
      </Link>
    </div>
  );
}

function BoardColumn({
  columnId,
  columnName,
  column,
  issues,
  agents,
  liveIssueIds,
  allIssues,
  onAddCard,
  onRenameColumn,
  onDeleteColumn,
}: {
  columnId: string;
  columnName: string;
  column?: BoardColumnType | null;
  issues: Issue[];
  agents?: Agent[];
  liveIssueIds?: Set<string>;
  allIssues: Issue[];
  onAddCard: (issueId: string, columnId: string) => void;
  onRenameColumn?: (columnId: string, name: string) => void;
  onDeleteColumn?: (columnId: string) => void;
}) {
  const { setNodeRef, isOver } = useDroppable({ id: columnId });
  const [addCardOpen, setAddCardOpen] = useState(false);
  const [addCardSearch, setAddCardSearch] = useState("");
  const [editingName, setEditingName] = useState(false);
  const [editNameValue, setEditNameValue] = useState(columnName);
  const isUnplaced = columnId === UNPLACED;
  const inColumnIds = new Set(issues.map((i) => i.id));
  const availableIssues = allIssues.filter((i) => !inColumnIds.has(i.id));
  const filteredAdd = addCardSearch.trim()
    ? availableIssues.filter(
        (i) =>
          i.title.toLowerCase().includes(addCardSearch.toLowerCase()) ||
          (i.identifier ?? i.id).toLowerCase().includes(addCardSearch.toLowerCase()),
      )
    : availableIssues.slice(0, 20);

  const handleRenameSubmit = () => {
    const trimmed = editNameValue.trim();
    if (trimmed && column && onRenameColumn) {
      onRenameColumn(column.id, trimmed);
      setEditingName(false);
    } else {
      setEditNameValue(columnName);
      setEditingName(false);
    }
  };

  return (
    <div className="flex flex-col min-w-[260px] w-[260px] shrink-0">
      <div className="flex items-center gap-1 px-2 py-2 mb-1 group">
        {editingName ? (
          <input
            autoFocus
            value={editNameValue}
            onChange={(e) => setEditNameValue(e.target.value)}
            onBlur={handleRenameSubmit}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleRenameSubmit();
              if (e.key === "Escape") {
                setEditNameValue(columnName);
                setEditingName(false);
              }
            }}
            className="flex-1 min-w-0 text-xs font-semibold uppercase tracking-wide text-muted-foreground bg-transparent border-b border-input outline-none px-0 py-0.5"
          />
        ) : (
          <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground truncate flex-1 min-w-0">
            {columnName}
          </span>
        )}
        <span className="text-xs text-muted-foreground/60 tabular-nums shrink-0">
          {issues.length}
        </span>
        {!isUnplaced && column && (onRenameColumn || onDeleteColumn) && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="icon-sm"
                className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                title="Column options"
              >
                <MoreHorizontal className="h-3.5 w-3.5" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start">
              {onRenameColumn && (
                <DropdownMenuItem onClick={() => { setEditNameValue(columnName); setEditingName(true); }}>
                  Rename column
                </DropdownMenuItem>
              )}
              {onDeleteColumn && (
                <DropdownMenuItem variant="destructive" onClick={() => onDeleteColumn(column.id)}>
                  Delete column
                </DropdownMenuItem>
              )}
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>
      <div
        ref={setNodeRef}
        className={`flex-1 min-h-[120px] rounded-md p-1 space-y-1 transition-colors ${
          isOver ? "bg-accent/40" : "bg-muted/20"
        }`}
      >
        <SortableContext
          items={issues.map((i) => i.id)}
          strategy={verticalListSortingStrategy}
        >
          {issues.map((issue) => (
            <KanbanColumnCard
              key={issue.id}
              issue={issue}
              agents={agents}
              isLive={liveIssueIds?.has(issue.id)}
            />
          ))}
        </SortableContext>
        <Popover open={addCardOpen} onOpenChange={setAddCardOpen}>
          <PopoverTrigger asChild>
            <button
              type="button"
              className="w-full mt-1 py-1.5 rounded text-xs text-muted-foreground hover:bg-muted/40 flex items-center justify-center gap-1"
              onClick={() => setAddCardOpen(true)}
            >
              <Plus className="h-3.5 w-3.5" />
              Add card
            </button>
          </PopoverTrigger>
          <PopoverContent className="w-72 p-1" align="start">
            <input
              type="text"
              placeholder="Search issues..."
              value={addCardSearch}
              onChange={(e) => setAddCardSearch(e.target.value)}
              className="w-full px-2 py-1.5 text-xs rounded border border-border bg-background mb-1"
            />
            <div className="max-h-56 overflow-y-auto">
              {filteredAdd.length === 0 ? (
                <p className="text-xs text-muted-foreground px-2 py-2">No issues to add</p>
              ) : (
                filteredAdd.map((issue) => (
                  <button
                    key={issue.id}
                    type="button"
                    className="w-full text-left px-2 py-1.5 text-xs rounded hover:bg-accent/50 flex items-center gap-2"
                    onClick={() => {
                      const position =
                        issues.length === 0
                          ? 1
                          : Math.max(...issues.map((i) => i.position ?? 0), 0) + 1;
                      onAddCard(issue.id, columnId);
                      setAddCardOpen(false);
                      setAddCardSearch("");
                    }}
                  >
                    <span className="text-muted-foreground font-mono shrink-0">
                      {issue.identifier ?? issue.id.slice(0, 8)}
                    </span>
                    <span className="truncate">{issue.title}</span>
                  </button>
                ))
              )}
            </div>
          </PopoverContent>
        </Popover>
      </div>
    </div>
  );
}

export function BoardKanban() {
  const { boardId } = useParams<{ boardId: string }>();
  const { selectedCompanyId } = useCompany();
  const { setBreadcrumbs } = useBreadcrumbs();
  const queryClient = useQueryClient();
  const [activeId, setActiveId] = useState<string | null>(null);
  const [newColumnName, setNewColumnName] = useState("");
  const [showAddColumn, setShowAddColumn] = useState(false);

  const { data: board, isLoading: boardLoading } = useQuery({
    queryKey: queryKeys.boards.detail(selectedCompanyId!, boardId!),
    queryFn: () => boardsApi.get(selectedCompanyId!, boardId!),
    enabled: !!selectedCompanyId && !!boardId,
  });

  const { data: columns = [], isLoading: columnsLoading } = useQuery({
    queryKey: queryKeys.boards.columns(selectedCompanyId!, boardId!),
    queryFn: () => boardsApi.listColumns(selectedCompanyId!, boardId!),
    enabled: !!selectedCompanyId && !!boardId,
  });

  const { data: allIssues = [] } = useQuery({
    queryKey: queryKeys.issues.list(selectedCompanyId!),
    queryFn: () => issuesApi.list(selectedCompanyId!),
    enabled: !!selectedCompanyId,
  });

  const { data: agents } = useQuery({
    queryKey: queryKeys.agents.list(selectedCompanyId!),
    queryFn: () => agentsApi.list(selectedCompanyId!),
    enabled: !!selectedCompanyId,
  });

  const { data: liveRuns } = useQuery({
    queryKey: queryKeys.liveRuns(selectedCompanyId!),
    queryFn: () => heartbeatsApi.liveRunsForCompany(selectedCompanyId!),
    enabled: !!selectedCompanyId,
    refetchInterval: 5000,
  });

  const updateIssue = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Record<string, unknown> }) =>
      issuesApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.issues.list(selectedCompanyId!) });
      queryClient.invalidateQueries({ queryKey: queryKeys.boards.columns(selectedCompanyId!, boardId!) });
    },
  });

  const createColumn = useMutation({
    mutationFn: (name: string) =>
      boardsApi.createColumn(selectedCompanyId!, boardId!, {
        name,
        position: columns.length,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.boards.columns(selectedCompanyId!, boardId!) });
      setNewColumnName("");
      setShowAddColumn(false);
    },
  });

  const updateColumn = useMutation({
    mutationFn: ({ columnId, name }: { columnId: string; name: string }) =>
      boardsApi.updateColumn(selectedCompanyId!, boardId!, columnId, { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.boards.columns(selectedCompanyId!, boardId!) });
    },
  });

  const removeColumn = useMutation({
    mutationFn: (columnId: string) =>
      boardsApi.removeColumn(selectedCompanyId!, boardId!, columnId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.boards.columns(selectedCompanyId!, boardId!) });
    },
  });

  const boardIssues = useMemo(
    () => allIssues.filter((i) => i.boardId === boardId),
    [allIssues, boardId]
  );

  const issuesByColumn = useMemo(() => {
    const map: Record<string, Issue[]> = {};
    map[UNPLACED] = [];
    for (const col of columns) {
      map[col.id] = [];
    }
    for (const issue of boardIssues) {
      const key = issue.boardColumnId ?? UNPLACED;
      if (map[key]) map[key].push(issue);
      else map[UNPLACED].push(issue);
    }
    for (const key of Object.keys(map)) {
      map[key].sort((a, b) => (a.position ?? 0) - (b.position ?? 0));
    }
    return map;
  }, [boardIssues, columns]);

  const liveIssueIds = useMemo(() => {
    const set = new Set<string>();
    for (const run of liveRuns ?? []) {
      if (run.issueId) set.add(run.issueId);
    }
    return set;
  }, [liveRuns]);

  const assigneeIdsOnBoard = useMemo(
    () =>
      boardIssues
        .map((i) => i.assigneeAgentId)
        .filter((id): id is string => !!id),
    [boardIssues]
  );

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  );

  const activeIssue = useMemo(
    () => (activeId ? boardIssues.find((i) => i.id === activeId) : null),
    [activeId, boardIssues]
  );

  useEffect(() => {
    setBreadcrumbs(
      board
        ? [{ label: "Boards", href: "/boards" }, { label: board.name }]
        : [{ label: "Boards", href: "/boards" }, { label: "…" }]
    );
  }, [setBreadcrumbs, board]);

  function handleDragStart(event: DragStartEvent) {
    setActiveId(event.active.id as string);
  }

  function handleDragEnd(event: DragEndEvent) {
    setActiveId(null);
    const { active, over } = event;
    if (!over) return;

    const issueId = active.id as string;
    const issue = boardIssues.find((i) => i.id === issueId);
    if (!issue) return;

    let targetColumnId: string | null = null;
    let newPosition: number | undefined;

    if (over.id === UNPLACED || columns.some((c) => c.id === over.id)) {
      targetColumnId = over.id === UNPLACED ? null : (over.id as string);
      const targetList = issuesByColumn[targetColumnId ?? UNPLACED] ?? [];
      const otherIds = targetList.map((i) => i.id).filter((id) => id !== issueId);
      if (otherIds.length === 0) {
        newPosition = 1;
      } else {
        newPosition = 0.5;
      }
    } else {
      const targetIssue = boardIssues.find((i) => i.id === over.id);
      if (targetIssue) {
        targetColumnId = targetIssue.boardColumnId ?? null;
        const list = issuesByColumn[targetColumnId ?? UNPLACED] ?? [];
        const idx = list.findIndex((i) => i.id === targetIssue.id);
        const prev = list[idx - 1];
        const next = list[idx];
        const prevPos = prev?.position ?? 0;
        const nextPos = next?.position ?? prevPos + 1;
        newPosition = (prevPos + nextPos) / 2;
      }
    }

    if (targetColumnId !== (issue.boardColumnId ?? null) || newPosition !== undefined) {
      updateIssue.mutate({
        id: issueId,
        data: {
          boardId: boardId!,
          boardColumnId: targetColumnId,
          position: newPosition,
        },
      });
    }
  }

  const addIssueToColumn = useCallback(
    (issueId: string, columnId: string) => {
      const list = issuesByColumn[columnId] ?? [];
      const position =
        list.length === 0 ? 1 : Math.max(...list.map((i) => i.position ?? 0), 0) + 1;
      updateIssue.mutate({
        id: issueId,
        data: {
          boardId: boardId!,
          boardColumnId: columnId === UNPLACED ? null : columnId,
          position,
        },
      });
    },
    [issuesByColumn, updateIssue, boardId],
  );

  if (!selectedCompanyId || !boardId) {
    return <EmptyState icon={LayoutGrid} message="Select a company and board." />;
  }

  if (boardLoading || columnsLoading) {
    return <PageSkeleton variant="list" />;
  }

  if (!board) {
    return <EmptyState icon={LayoutGrid} message="Board not found." />;
  }

  const columnList = [
    { id: UNPLACED, name: "Unplaced", position: -1 },
    ...columns.slice().sort((a, b) => a.position - b.position),
  ];

  return (
    <div className="flex flex-col min-h-0">
      <BoardHeader
        boardName={board.name}
        assigneeIds={assigneeIdsOnBoard}
        agents={agents}
        onShare={() => {
          const url = window.location.href;
          void navigator.clipboard.writeText(url).then(() => { /* optional toast */ });
        }}
      />

      <DndContext
        sensors={sensors}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
      >
        <div className="flex gap-3 overflow-x-auto pb-4 -mx-2 px-2 flex-1 min-h-0">
          {columnList.map((col) => (
            <BoardColumn
              key={col.id}
              columnId={col.id}
              columnName={col.name}
              column={col.id === UNPLACED ? undefined : (col as BoardColumnType)}
              issues={issuesByColumn[col.id] ?? []}
              agents={agents}
              liveIssueIds={liveIssueIds}
              allIssues={allIssues}
              onAddCard={addIssueToColumn}
              onRenameColumn={
                selectedCompanyId && boardId
                  ? (columnId, name) => updateColumn.mutate({ columnId, name })
                  : undefined
              }
              onDeleteColumn={
                selectedCompanyId && boardId
                  ? (columnId) => removeColumn.mutate(columnId)
                  : undefined
              }
            />
          ))}
          <div className="min-w-[260px] w-[260px] shrink-0 flex flex-col">
            {showAddColumn ? (
              <div className="rounded-md border border-dashed border-border bg-muted/10 p-2">
                <input
                  type="text"
                  placeholder="Column name"
                  value={newColumnName}
                  onChange={(e) => setNewColumnName(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && newColumnName.trim()) {
                      createColumn.mutate(newColumnName.trim());
                    }
                    if (e.key === "Escape") setShowAddColumn(false);
                  }}
                  autoFocus
                  className="w-full h-9 rounded-md border border-border bg-background px-3 text-sm mb-2"
                />
                <div className="flex gap-1">
                  <Button
                    size="sm"
                    onClick={() => {
                      if (newColumnName.trim()) createColumn.mutate(newColumnName.trim());
                    }}
                    disabled={!newColumnName.trim() || createColumn.isPending}
                  >
                    Add
                  </Button>
                  <Button size="sm" variant="ghost" onClick={() => setShowAddColumn(false)}>
                    Cancel
                  </Button>
                </div>
              </div>
            ) : (
              <button
                type="button"
                onClick={() => setShowAddColumn(true)}
                className="flex items-center gap-2 min-h-[120px] rounded-md border border-dashed border-border bg-muted/10 p-3 text-sm text-muted-foreground hover:bg-muted/20 transition-colors"
              >
                <Plus className="h-4 w-4" />
                Add column
              </button>
            )}
          </div>
        </div>
        <DragOverlay>
          {activeIssue ? (
            <KanbanColumnCard
              issue={activeIssue}
              agents={agents}
              isOverlay
            />
          ) : null}
        </DragOverlay>
      </DndContext>

      <nav className="sticky bottom-0 z-10 flex items-center gap-1 px-4 py-2 border-t border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80">
        <Button variant="ghost" size="sm" asChild>
          <Link to="/inbox" className="gap-1.5">
            <Inbox className="h-4 w-4" />
            Inbox
          </Link>
        </Button>
        <Button variant="ghost" size="sm" asChild>
          <Link to="/issues" className="gap-1.5">
            <Calendar className="h-4 w-4" />
            Planner
          </Link>
        </Button>
        <Button variant="secondary" size="sm" className="gap-1.5 pointer-events-none" aria-current="page">
          <LayoutGrid className="h-4 w-4" />
          Board
        </Button>
        <Button variant="ghost" size="sm" asChild className="ml-auto">
          <Link to="/boards" className="gap-1.5">
            <LayoutGrid className="h-4 w-4" />
            Switch boards
          </Link>
        </Button>
      </nav>
    </div>
  );
}
