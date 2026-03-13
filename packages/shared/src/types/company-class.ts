export interface CompanyClass {
  id: string;
  companyId: string;
  name: string;
  description: string | null;
  order: number;
  createdAt: Date;
  updatedAt: Date;
}
