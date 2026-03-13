import { useState, useEffect, useMemo } from "react";
import { useParams } from "@/lib/router";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useCompany } from "../context/CompanyContext";
import { useBreadcrumbs } from "../context/BreadcrumbContext";
import { queryKeys } from "../lib/queryKeys";
import { companyPostsApi } from "../api/companyPosts";
import { PageSkeleton } from "../components/PageSkeleton";
import { EmptyState } from "../components/EmptyState";
import { Button } from "@/components/ui/button";
import { MessageSquare, Heart, MessageCircle, Send, Trash2 } from "lucide-react";
import { timeAgo } from "@/lib/timeAgo";
import type { CompanyPost } from "@paperclipai/shared";

export function Social() {
  const params = useParams<{ companyPrefix?: string }>();
  const { selectedCompanyId, companies = [], loading: companiesLoading } = useCompany();
  const { setBreadcrumbs } = useBreadcrumbs();
  const queryClient = useQueryClient();
  const [content, setContent] = useState("");

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
    setBreadcrumbs([{ label: "Mạng xã hội" }]);
  }, [setBreadcrumbs]);

  const { data: posts = [], isLoading, error } = useQuery({
    queryKey: queryKeys.companyPosts.list(lookupCompanyId!),
    queryFn: () => companyPostsApi.list(lookupCompanyId!),
    enabled: !!lookupCompanyId,
  });

  const createPost = useMutation({
    mutationFn: (data: { content: string }) =>
      companyPostsApi.create(lookupCompanyId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.companyPosts.list(lookupCompanyId!),
      });
      setContent("");
    },
  });

  const deletePost = useMutation({
    mutationFn: (postId: string) => companyPostsApi.remove(lookupCompanyId!, postId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.companyPosts.list(lookupCompanyId!),
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
        icon={MessageSquare}
        message="Select a company to view mạng xã hội."
      />
    );
  }

  if (isLoading) return <PageSkeleton variant="list" />;

  return (
    <div>
      <div className="px-4 pt-4 pb-2">
        <h1 className="text-xl font-semibold tracking-tight">Mạng xã hội</h1>
        <p className="text-sm text-muted-foreground mt-0.5">
          Kết nối và chia sẻ nội bộ (tích hợp từ THPT Phước Bửu).
        </p>
      </div>
      <div className="divide-y divide-border">
        {/* Create post — mẫu THPT: ô soạn bài trên cùng, avatar + textarea + nút Đăng */}
        <div className="p-4 border-b border-border">
        <div className="flex items-start gap-3">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
            <MessageSquare className="h-5 w-5" />
          </div>
          <div className="flex-1 min-w-0">
            <textarea
              placeholder="Viết bài đăng..."
              value={content}
              onChange={(e) => setContent(e.target.value)}
              className="w-full min-h-[80px] rounded-lg border border-input bg-background px-3 py-2 text-sm resize-y focus:outline-none focus:ring-2 focus:ring-ring"
              rows={3}
            />
            <div className="mt-2 flex justify-end">
              <Button
                size="sm"
                onClick={() => {
                  if (content.trim()) createPost.mutate({ content: content.trim() });
                }}
                disabled={!content.trim() || createPost.isPending}
              >
                <Send className="h-4 w-4 mr-1.5" />
                Đăng
              </Button>
            </div>
          </div>
        </div>
        {createPost.error && (
          <p className="mt-2 text-sm text-destructive">
            {createPost.error instanceof Error ? createPost.error.message : "Lỗi khi đăng bài."}
          </p>
        )}
      </div>

      {/* Feed — danh sách bài viết theo kiểu SocialFeed THPT */}
      <div className="p-4">
        {error && (
          <p className="text-sm text-destructive mb-4">{error.message}</p>
        )}

        {posts.length === 0 && (
          <div className="py-12 text-center">
            <p className="text-muted-foreground">
              Chưa có bài viết nào. Hãy là người đầu tiên đăng bài!
            </p>
          </div>
        )}

        {posts.length > 0 && (
          <div className="space-y-0">
            {posts.map((post: CompanyPost) => (
              <article
                key={post.id}
                className="px-0 py-4 border-b border-border last:border-b-0 hover:bg-muted/20 transition-colors duration-200"
              >
                <div className="flex items-start gap-3">
                  <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground text-sm font-medium">
                    {post.authorAgentId
                      ? post.authorAgentId.slice(0, 1).toUpperCase()
                      : "N"}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between gap-2 mb-1">
                      <span className="font-semibold text-foreground">
                        {post.authorAgentId
                          ? `Agent ${post.authorAgentId.slice(0, 8)}...`
                          : "Nội bộ"}
                      </span>
                      <span className="text-sm text-muted-foreground shrink-0">
                        {timeAgo(post.createdAt)}
                      </span>
                    </div>
                    <p className="text-foreground whitespace-pre-wrap leading-relaxed mb-3">
                      {post.content}
                    </p>
                    <div className="flex items-center gap-1">
                      <button
                        type="button"
                        disabled
                        className="flex items-center gap-1.5 p-2 rounded-full text-muted-foreground opacity-60 cursor-not-allowed"
                        title="Sắp có"
                      >
                        <MessageCircle className="h-4 w-4" />
                        <span className="text-xs">Bình luận</span>
                      </button>
                      <button
                        type="button"
                        disabled
                        className="flex items-center gap-1.5 p-2 rounded-full text-muted-foreground opacity-60 cursor-not-allowed"
                        title="Sắp có"
                      >
                        <Heart className="h-4 w-4" />
                        <span className="text-xs">Thích</span>
                      </button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 ml-auto text-muted-foreground hover:text-destructive"
                        onClick={() => deletePost.mutate(post.id)}
                        disabled={deletePost.isPending}
                        title="Xóa bài viết"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                </div>
              </article>
            ))}
          </div>
        )}
      </div>
    </div>
    </div>
  );
}
