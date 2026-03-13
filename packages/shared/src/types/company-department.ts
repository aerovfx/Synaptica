export interface CompanyDepartment {
  id: string;
  companyId: string;
  spaceId: string | null;
  name: string;
  leaderAgentId: string | null;
  order: number;
  createdAt: Date;
  updatedAt: Date;
}
