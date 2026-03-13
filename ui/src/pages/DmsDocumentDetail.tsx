import { useParams, useNavigate } from "@/lib/router";
import { Button } from "@/components/ui/button";
import { ArrowLeft } from "lucide-react";

type Kind = "incoming" | "outgoing";

export function DmsDocumentDetail({ kind }: { kind: Kind }) {
  const params = useParams<{ documentId: string }>();
  const navigate = useNavigate();
  const documentId = params.documentId;
  const label = kind === "incoming" ? "Văn bản đến" : "Văn bản đi";

  return (
    <div className="p-6 flex flex-col gap-4">
      <Button variant="ghost" size="sm" className="w-fit" onClick={() => navigate(-1)}>
        <ArrowLeft className="h-4 w-4 mr-1" />
        Quay lại
      </Button>
      <div className="rounded-lg border bg-card p-6">
        <h1 className="text-lg font-semibold mb-2">Chi tiết {label}</h1>
        <p className="text-sm text-muted-foreground">
          Mã văn bản: <code className="bg-muted px-1 rounded">{documentId}</code>
        </p>
        <p className="text-sm text-muted-foreground mt-2">
          Trang chi tiết đầy đủ sẽ được bổ sung khi backend DMS có API tương ứng.
        </p>
      </div>
    </div>
  );
}
