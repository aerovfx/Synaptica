import { pgTable, uuid, text, integer, timestamp, date, index } from "drizzle-orm/pg-core";
import { companies } from "./companies.js";
import { agents } from "./agents.js";

/** Tài liệu chung (public documents) — company-scoped. */
export const companyDmsDocuments = pgTable(
  "company_dms_documents",
  {
    id: uuid("id").primaryKey().defaultRandom(),
    companyId: uuid("company_id").notNull().references(() => companies.id),
    title: text("title").notNull(),
    description: text("description"),
    type: text("type").notNull().default("OTHER"),
    fileSize: integer("file_size").notNull().default(0),
    fileUrl: text("file_url").notNull().default(""),
    uploadedByAgentId: uuid("uploaded_by_agent_id").references(() => agents.id),
    createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
  },
  (table) => ({
    companyIdx: index("company_dms_documents_company_idx").on(table.companyId),
  }),
);

/** Văn bản đến — company-scoped. */
export const companyDmsIncoming = pgTable(
  "company_dms_incoming",
  {
    id: uuid("id").primaryKey().defaultRandom(),
    companyId: uuid("company_id").notNull().references(() => companies.id),
    title: text("title").notNull(),
    documentNumber: text("document_number"),
    type: text("type").notNull().default("OTHER"),
    status: text("status").notNull().default("pending"),
    priority: text("priority").notNull().default("normal"),
    sender: text("sender"),
    receivedDate: date("received_date", { mode: "string" }).notNull(),
    deadline: date("deadline", { mode: "string" }),
    summary: text("summary"),
    createdByAgentId: uuid("created_by_agent_id").references(() => agents.id),
    createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
  },
  (table) => ({
    companyIdx: index("company_dms_incoming_company_idx").on(table.companyId),
    receivedDateIdx: index("company_dms_incoming_received_date_idx").on(table.receivedDate),
  }),
);

/** Văn bản đi — company-scoped. */
export const companyDmsOutgoing = pgTable(
  "company_dms_outgoing",
  {
    id: uuid("id").primaryKey().defaultRandom(),
    companyId: uuid("company_id").notNull().references(() => companies.id),
    title: text("title").notNull(),
    documentNumber: text("document_number"),
    status: text("status").notNull().default("draft"),
    priority: text("priority").notNull().default("normal"),
    recipient: text("recipient"),
    createdByAgentId: uuid("created_by_agent_id").references(() => agents.id),
    createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
  },
  (table) => ({
    companyIdx: index("company_dms_outgoing_company_idx").on(table.companyId),
  }),
);
