import { pgTable, uuid, text, timestamp, index } from "drizzle-orm/pg-core";
import { companies } from "./companies.js";
import { agents } from "./agents.js";

export const companyPosts = pgTable(
  "company_posts",
  {
    id: uuid("id").primaryKey().defaultRandom(),
    companyId: uuid("company_id").notNull().references(() => companies.id),
    authorAgentId: uuid("author_agent_id").references(() => agents.id),
    content: text("content").notNull(),
    scheduledAt: timestamp("scheduled_at", { withTimezone: true }),
    createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
    updatedAt: timestamp("updated_at", { withTimezone: true }).notNull().defaultNow(),
  },
  (table) => ({
    companyIdx: index("company_posts_company_idx").on(table.companyId),
    createdAtIdx: index("company_posts_created_at_idx").on(table.createdAt),
  }),
);
