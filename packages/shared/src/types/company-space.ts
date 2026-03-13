export interface CompanySpace {
  id: string;
  companyId: string;
  parentId: string | null;
  name: string;
  order: number;
  createdAt: Date;
  updatedAt: Date;
}
