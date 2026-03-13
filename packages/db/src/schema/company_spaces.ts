import { pgTable, uuid, text, integer, timestamp, index } from "drizzle-orm/pg-core";
import { companies } from "./companies.js";

export const companySpaces = pgTable(
  "company_spaces",
  {
    id: uuid("id").primaryKey().defaultRandom(),
    companyId: uuid("company_id").notNull().references(() => companies.id),
    parentId: uuid("parent_id"),
    name: text("name").notNull(),
    order: integer("order").notNull().default(0),
    createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
    updatedAt: timestamp("updated_at", { withTimezone: true }).notNull().defaultNow(),
  },
  (table) => ({
    companyIdx: index("company_spaces_company_idx").on(table.companyId),
    parentIdx: index("company_spaces_parent_idx").on(table.parentId),
  }),
);
