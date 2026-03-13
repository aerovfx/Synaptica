import type { LucideIcon } from "lucide-react";

export function PlaceholderPage({
  title,
  description,
  icon: Icon,
}: {
  title: string;
  description?: string;
  icon: LucideIcon;
}) {
  return (
    <div className="flex flex-col gap-4 p-6">
      <h1 className="text-2xl font-semibold tracking-tight flex items-center gap-2">
        {Icon && <Icon className="h-7 w-7 text-muted-foreground" />}
        {title}
      </h1>
      {description && (
        <p className="text-muted-foreground">{description}</p>
      )}
      <div className="rounded-lg border border-dashed border-border bg-muted/30 p-8 text-center text-sm text-muted-foreground">
        Trang đang được phát triển. Nội dung sẽ được bổ sung sau.
      </div>
    </div>
  );
}
