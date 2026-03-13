export interface CompanyPost {
  id: string;
  companyId: string;
  authorAgentId: string | null;
  content: string;
  scheduledAt: Date | null;
  createdAt: Date;
  updatedAt: Date;
}
