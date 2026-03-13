import { pgTable, uuid, text, integer, timestamp, index } from "drizzle-orm/pg-core";
import { companies } from "./companies.js";
import { companySpaces } from "./company_spaces.js";
import { agents } from "./agents.js";

export const companyDepartments = pgTable(
  "company_departments",
  {
    id: uuid("id").primaryKey().defaultRandom(),
    companyId: uuid("company_id").notNull().references(() => companies.id),
    spaceId: uuid("space_id").references(() => companySpaces.id),
    name: text("name").notNull(),
    leaderAgentId: uuid("leader_agent_id").references(() => agents.id),
    order: integer("order").notNull().default(0),
    createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
    updatedAt: timestamp("updated_at", { withTimezone: true }).notNull().defaultNow(),
  },
  (table) => ({
    companyIdx: index("company_departments_company_idx").on(table.companyId),
    spaceIdx: index("company_departments_space_idx").on(table.spaceId),
  }),
);
