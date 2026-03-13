import { pgTable, uuid, text, timestamp, date, index } from "drizzle-orm/pg-core";
import { boards } from "./boards.js";

export const sprints = pgTable(
  "sprints",
  {
    id: uuid("id").primaryKey().defaultRandom(),
    boardId: uuid("board_id").notNull().references(() => boards.id, { onDelete: "cascade" }),
    name: text("name").notNull(),
    startDate: date("start_date"),
    endDate: date("end_date"),
    status: text("status").notNull().default("planned"), // planned | active | completed
    createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
    updatedAt: timestamp("updated_at", { withTimezone: true }).notNull().defaultNow(),
  },
  (table) => ({
    boardIdx: index("sprints_board_idx").on(table.boardId),
  }),
);
