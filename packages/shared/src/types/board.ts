export interface Board {
  id: string;
  companyId: string;
  projectId: string | null;
  name: string;
  type: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface BoardColumn {
  id: string;
  boardId: string;
  name: string;
  position: number;
  createdAt: Date;
  updatedAt: Date;
}

export interface Sprint {
  id: string;
  boardId: string;
  name: string;
  startDate: string | null;
  endDate: string | null;
  status: string;
  createdAt: Date;
  updatedAt: Date;
}
