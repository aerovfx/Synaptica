/**
 * DMS (Document Management System) API — Văn bản.
 * Company-scoped. Backend may return 404 until DMS routes exist; UI falls back to empty lists.
 */

import { api } from "./client";

export interface DmsUser {
  id: string;
  firstName: string;
  lastName: string;
  avatar?: string | null;
}

export interface DmsDocumentPublic {
  id: string;
  title: string;
  description?: string | null;
  type: string;
  fileSize: number;
  fileUrl: string;
  createdAt: string;
  uploadedBy: DmsUser & { id?: string };
}

export interface DmsIncomingDocument {
  id: string;
  title: string;
  documentNumber: string | null;
  type: string;
  status: string;
  priority: string;
  sender: string | null;
  receivedDate: string;
  deadline: string | null;
  summary: string | null;
  assignments: Array<{
    id: string;
    assignedTo: DmsUser;
    status: string;
    deadline: string | null;
  }>;
  createdAt: string;
  createdBy?: DmsUser;
}

export interface DmsOutgoingDocument {
  id: string;
  title: string;
  documentNumber: string | null;
  status: string;
  priority: string;
  recipient: string | null;
  createdAt: string;
  createdBy: DmsUser;
}

/** Combined response from GET /companies/:id/dms — one round-trip for all DMS data. */
export interface DmsListAllResponse {
  documents: DmsDocumentPublic[];
  incoming: DmsIncomingDocument[];
  outgoing: DmsOutgoingDocument[];
}

async function getOrEmpty<T>(fn: () => Promise<T>, fallback: T): Promise<T> {
  try {
    return await fn();
  } catch (e: unknown) {
    const err = e as { status?: number };
    if (err?.status === 404 || err?.status === 501 || err?.status === 503) return fallback;
    throw e;
  }
}

/** Payload for upload (base64 file + metadata). */
export interface UploadDmsDocumentPayload {
  title?: string;
  description?: string | null;
  type?: string;
  contentBase64: string;
  contentType: string;
  fileName: string;
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = reader.result as string;
      const base64 = dataUrl.split(",")[1];
      resolve(base64 ?? "");
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

export const dmsApi = {
  /** Single request for documents + incoming + outgoing (faster page load). */
  listAll: (companyId: string) =>
    getOrEmpty(
      () => api.get<DmsListAllResponse>(`/companies/${companyId}/dms`),
      { documents: [], incoming: [], outgoing: [] },
    ),
  /** Upload file as tài liệu chung (creates asset + DMS document). */
  uploadDocument: async (
    companyId: string,
    file: File,
    meta?: { title?: string; description?: string | null; type?: string }
  ): Promise<DmsDocumentPublic> => {
    const contentBase64 = await fileToBase64(file);
    const payload: UploadDmsDocumentPayload = {
      contentBase64,
      contentType: file.type || "application/octet-stream",
      fileName: file.name,
      title: meta?.title?.trim() || undefined,
      description: meta?.description?.trim() || undefined,
      type: meta?.type || "OTHER",
    };
    return api.post<DmsDocumentPublic>(
      `/companies/${companyId}/dms/documents/upload`,
      payload
    );
  },
  listPublic: (companyId: string) =>
    getOrEmpty(
      () => api.get<DmsDocumentPublic[]>(`/companies/${companyId}/dms/documents`),
      [],
    ),
  listIncoming: (companyId: string) =>
    getOrEmpty(
      () => api.get<DmsIncomingDocument[]>(`/companies/${companyId}/dms/incoming`),
      [],
    ),
  listOutgoing: (companyId: string) =>
    getOrEmpty(
      () => api.get<DmsOutgoingDocument[]>(`/companies/${companyId}/dms/outgoing`),
      [],
    ),
};
