import { useState, useEffect, useMemo } from "react";
import { FileText, Inbox, Send, Download, Upload } from "lucide-react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useParams } from "@/lib/router";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useCompany } from "@/context/CompanyContext";
import { useBreadcrumbs } from "@/context/BreadcrumbContext";
import { useNavigate } from "@/lib/router";
import { queryKeys } from "@/lib/queryKeys";
import { dmsApi, type DmsDocumentPublic, type DmsIncomingDocument, type DmsOutgoingDocument } from "@/api/dms";
import { timeAgo } from "@/lib/timeAgo";
import { PageSkeleton } from "@/components/PageSkeleton";
import { EmptyState } from "@/components/EmptyState";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

const DOCUMENT_TYPE_LABELS: Record<string, string> = {
  ANNOUNCEMENT: "Thông báo",
  POLICY: "Chính sách",
  REPORT: "Báo cáo",
  FORM: "Biểu mẫu",
  OTHER: "Khác",
};

export function Dms() {
  const params = useParams<{ companyPrefix?: string }>();
  const { selectedCompanyId, companies = [], loading: companiesLoading } = useCompany();
  const { setBreadcrumbs } = useBreadcrumbs();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState("all");
  const [showUpload, setShowUpload] = useState(false);
  const [uploadFile, setUploadFile] = useState<File | null>(null);
  const [uploadTitle, setUploadTitle] = useState("");
  const [uploadDescription, setUploadDescription] = useState("");
  const [uploadType, setUploadType] = useState<string>("OTHER");

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
    setBreadcrumbs([{ label: "Văn bản" }]);
  }, [setBreadcrumbs]);

  const { data: dmsData, isLoading: loading } = useQuery({
    queryKey: queryKeys.dms.all(lookupCompanyId!),
    queryFn: () => dmsApi.listAll(lookupCompanyId!),
    enabled: !!lookupCompanyId,
  });

  const docs = dmsData?.documents ?? [];
  const inc = dmsData?.incoming ?? [];
  const out = dmsData?.outgoing ?? [];
  const totalCount = docs.length + inc.length + out.length;

  const uploadDocument = useMutation({
    mutationFn: (file: File) =>
      dmsApi.uploadDocument(lookupCompanyId!, file, {
        title: uploadTitle.trim() || undefined,
        description: uploadDescription.trim() || undefined,
        type: uploadType,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.dms.all(lookupCompanyId!) });
      setUploadFile(null);
      setUploadTitle("");
      setUploadDescription("");
      setUploadType("OTHER");
      setShowUpload(false);
      setActiveTab("public");
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
          Không tìm thấy công ty với prefix &quot;{companyPrefix}&quot;. Kiểm tra URL hoặc chọn công ty từ sidebar.
        </p>
      </div>
    );
  }

  if (!lookupCompanyId) {
    if (companyPrefix || companiesLoading) return <PageSkeleton variant="list" />;
    return (
      <div className="p-6 text-muted-foreground">
        Chọn công ty để xem văn bản.
      </div>
    );
  }

  if (loading) {
    return <PageSkeleton variant="list" />;
  }

  return (
    <div className="flex flex-col gap-6 p-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <h1 className="text-2xl font-semibold tracking-tight">Văn bản</h1>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setShowUpload((v) => !v)}
          className="shrink-0"
        >
          <Upload className="h-4 w-4 mr-1.5" />
          Tải lên tài liệu
        </Button>
      </div>

      {showUpload && (
        <div className="rounded-lg border border-border bg-card p-4 space-y-4">
          <h3 className="text-sm font-medium">Tải lên tài liệu chung</h3>
          <div className="grid gap-3 sm:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="dms-file">File</Label>
              <Input
                id="dms-file"
                type="file"
                onChange={(e) => {
                  const f = e.target.files?.[0];
                  setUploadFile(f ?? null);
                  if (f && !uploadTitle.trim()) setUploadTitle(f.name);
                }}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="dms-upload-title">Tiêu đề (tùy chọn)</Label>
              <Input
                id="dms-upload-title"
                placeholder="Để trống sẽ dùng tên file"
                value={uploadTitle}
                onChange={(e) => setUploadTitle(e.target.value)}
              />
            </div>
            <div className="space-y-2 sm:col-span-2">
              <Label htmlFor="dms-upload-desc">Mô tả (tùy chọn)</Label>
              <Input
                id="dms-upload-desc"
                placeholder="Mô tả ngắn"
                value={uploadDescription}
                onChange={(e) => setUploadDescription(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>Loại tài liệu</Label>
              <Select value={uploadType} onValueChange={setUploadType}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {Object.entries(DOCUMENT_TYPE_LABELS).map(([value, label]) => (
                    <SelectItem key={value} value={value}>
                      {label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="flex gap-2">
            <Button
              size="sm"
              disabled={!uploadFile || uploadDocument.isPending}
              onClick={() => uploadFile && uploadDocument.mutate(uploadFile)}
            >
              {uploadDocument.isPending ? "Đang tải lên…" : "Tải lên"}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                setShowUpload(false);
                setUploadFile(null);
                setUploadTitle("");
                setUploadDescription("");
              }}
            >
              Hủy
            </Button>
          </div>
          {uploadDocument.isError && (
            <p className="text-sm text-destructive">
              {uploadDocument.error instanceof Error
                ? uploadDocument.error.message
                : "Lỗi khi tải lên."}
            </p>
          )}
        </div>
      )}

      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList variant="line" className="w-full justify-start border-b rounded-none h-auto p-0 bg-transparent">
          <TabsTrigger value="all" className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary">
            <FileText className="h-4 w-4 mr-1.5" />
            Tất cả
            <Badge variant="secondary" className="ml-1.5 text-xs">
              {totalCount}
            </Badge>
          </TabsTrigger>
          <TabsTrigger value="public" className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary">
            <FileText className="h-4 w-4 mr-1.5" />
            Tài liệu chung
            <Badge variant="secondary" className="ml-1.5 text-xs">
              {docs.length}
            </Badge>
          </TabsTrigger>
          <TabsTrigger value="incoming" className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary">
            <Inbox className="h-4 w-4 mr-1.5" />
            Văn bản đến
            <Badge variant="secondary" className="ml-1.5 text-xs">
              {inc.length}
            </Badge>
          </TabsTrigger>
          <TabsTrigger value="outgoing" className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary">
            <Send className="h-4 w-4 mr-1.5" />
            Văn bản đi
            <Badge variant="secondary" className="ml-1.5 text-xs">
              {out.length}
            </Badge>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="all" className="mt-6">
          <DmsAllTab
            documents={docs}
            incoming={inc}
            outgoing={out}
            onIncomingClick={(id) => navigate(`/dms/incoming/${id}`)}
            onOutgoingClick={(id) => navigate(`/dms/outgoing/${id}`)}
          />
        </TabsContent>
        <TabsContent value="public" className="mt-6">
          <DmsPublicTab documents={docs} />
        </TabsContent>
        <TabsContent value="incoming" className="mt-6">
          <DmsIncomingTab documents={inc} onDocumentClick={(id) => navigate(`/dms/incoming/${id}`)} />
        </TabsContent>
        <TabsContent value="outgoing" className="mt-6">
          <DmsOutgoingTab documents={out} onDocumentClick={(id) => navigate(`/dms/outgoing/${id}`)} />
        </TabsContent>
      </Tabs>
    </div>
  );
}

function DmsAllTab({
  documents,
  incoming,
  outgoing,
  onIncomingClick,
  onOutgoingClick,
}: {
  documents: DmsDocumentPublic[];
  incoming: DmsIncomingDocument[];
  outgoing: DmsOutgoingDocument[];
  onIncomingClick: (id: string) => void;
  onOutgoingClick: (id: string) => void;
}) {
  const docs = documents ?? [];
  const inc = incoming ?? [];
  const out = outgoing ?? [];
  const hasAny = docs.length > 0 || inc.length > 0 || out.length > 0;
  if (!hasAny) {
    return (
      <EmptyState
        icon={FileText}
        message="Chưa có văn bản nào. Văn bản đến, văn bản đi và tài liệu chung sẽ hiển thị tại đây."
      />
    );
  }
  return (
    <div className="space-y-8">
      {docs.length > 0 && (
        <section>
          <h2 className="text-lg font-medium mb-4 flex items-center gap-2">
            <FileText className="h-5 w-5" />
            Tài liệu chung ({docs.length})
          </h2>
          <div className="rounded-lg border bg-card divide-y">
            {docs.map((doc) => (
              <DmsPublicRow key={doc.id} doc={doc} />
            ))}
          </div>
        </section>
      )}
      {inc.length > 0 && (
        <section>
          <h2 className="text-lg font-medium mb-4 flex items-center gap-2">
            <Inbox className="h-5 w-5" />
            Văn bản đến ({inc.length})
          </h2>
          <div className="rounded-lg border bg-card divide-y">
            {inc.map((doc) => (
              <div
                key={doc.id}
                className="p-4 hover:bg-muted/50 cursor-pointer transition-colors"
                onClick={() => onIncomingClick(doc.id)}
              >
                <DmsIncomingRow doc={doc} />
              </div>
            ))}
          </div>
        </section>
      )}
      {out.length > 0 && (
        <section>
          <h2 className="text-lg font-medium mb-4 flex items-center gap-2">
            <Send className="h-5 w-5" />
            Văn bản đi ({out.length})
          </h2>
          <div className="rounded-lg border bg-card divide-y">
            {out.map((doc) => (
              <div
                key={doc.id}
                className="p-4 hover:bg-muted/50 cursor-pointer transition-colors"
                onClick={() => onOutgoingClick(doc.id)}
              >
                <DmsOutgoingRow doc={doc} />
              </div>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}

function DmsPublicRow({ doc }: { doc: DmsDocumentPublic }) {
  const name = doc.uploadedBy ? `${doc.uploadedBy.firstName} ${doc.uploadedBy.lastName}` : "—";
  return (
    <div className="p-4 flex items-start justify-between gap-4">
      <div className="min-w-0 flex-1">
        <p className="font-medium truncate">{doc.title}</p>
        {doc.description && <p className="text-sm text-muted-foreground mt-0.5 line-clamp-1">{doc.description}</p>}
        <p className="text-xs text-muted-foreground mt-1">
          Đăng bởi: {name} · {timeAgo(doc.createdAt)} · {formatFileSize(doc.fileSize)}
        </p>
      </div>
      {doc.fileUrl && (
        <Button variant="ghost" size="icon" asChild>
          <a href={doc.fileUrl} target="_blank" rel="noopener noreferrer" title="Tải xuống">
            <Download className="h-4 w-4" />
          </a>
        </Button>
      )}
    </div>
  );
}

function DmsPublicTab({ documents }: { documents: DmsDocumentPublic[] }) {
  const list = documents ?? [];
  if (list.length === 0) {
    return (
      <EmptyState
        icon={FileText}
        message="Chưa có tài liệu chung. Tài liệu chung được đăng để mọi người xem và tải."
      />
    );
  }
  return (
    <div className="rounded-lg border bg-card divide-y">
      {list.map((doc) => (
        <DmsPublicRow key={doc.id} doc={doc} />
      ))}
    </div>
  );
}

function DmsIncomingRow({ doc }: { doc: DmsIncomingDocument }) {
  return (
    <>
      <div className="flex items-center gap-2 mb-1">
        <Badge variant="secondary" className="text-xs">
          {doc.status}
        </Badge>
        <Badge variant="outline" className="text-xs">
          {doc.priority}
        </Badge>
      </div>
      <p className="font-medium">{doc.title}</p>
      {doc.documentNumber && <p className="text-sm text-muted-foreground">Số: {doc.documentNumber}</p>}
      {doc.sender && <p className="text-sm text-muted-foreground">Người gửi: {doc.sender}</p>}
      <p className="text-xs text-muted-foreground mt-1">{timeAgo(doc.createdAt)}</p>
    </>
  );
}

function DmsIncomingTab({
  documents,
  onDocumentClick,
}: {
  documents: DmsIncomingDocument[];
  onDocumentClick: (id: string) => void;
}) {
  const list = documents ?? [];
  if (list.length === 0) {
    return (
      <EmptyState
        icon={Inbox}
        message="Chưa có văn bản đến. Văn bản đến từ bên ngoài sẽ hiển thị tại đây."
      />
    );
  }
  return (
    <div className="rounded-lg border bg-card divide-y">
      {list.map((doc) => (
        <div
          key={doc.id}
          className="p-4 hover:bg-muted/50 cursor-pointer transition-colors"
          onClick={() => onDocumentClick(doc.id)}
        >
          <DmsIncomingRow doc={doc} />
        </div>
      ))}
    </div>
  );
}

function DmsOutgoingRow({ doc }: { doc: DmsOutgoingDocument }) {
  const name = doc.createdBy ? `${doc.createdBy.firstName} ${doc.createdBy.lastName}` : "—";
  return (
    <>
      <div className="flex items-center gap-2 mb-1">
        <Badge variant="secondary" className="text-xs">
          {doc.status}
        </Badge>
        <Badge variant="outline" className="text-xs">
          {doc.priority}
        </Badge>
      </div>
      <p className="font-medium">{doc.title}</p>
      {doc.documentNumber && <p className="text-sm text-muted-foreground">Số: {doc.documentNumber}</p>}
      {doc.recipient && <p className="text-sm text-muted-foreground">Người nhận: {doc.recipient}</p>}
      <p className="text-xs text-muted-foreground mt-1">Tạo bởi: {name} · {timeAgo(doc.createdAt)}</p>
    </>
  );
}

function DmsOutgoingTab({
  documents,
  onDocumentClick,
}: {
  documents: DmsOutgoingDocument[];
  onDocumentClick: (id: string) => void;
}) {
  const list = documents ?? [];
  if (list.length === 0) {
    return (
      <EmptyState
        icon={Send}
        message="Chưa có văn bản đi. Văn bản đi do công ty tạo sẽ hiển thị tại đây."
      />
    );
  }
  return (
    <div className="rounded-lg border bg-card divide-y">
      {list.map((doc) => (
        <div
          key={doc.id}
          className="p-4 hover:bg-muted/50 cursor-pointer transition-colors"
          onClick={() => onDocumentClick(doc.id)}
        >
          <DmsOutgoingRow doc={doc} />
        </div>
      ))}
    </div>
  );
}
