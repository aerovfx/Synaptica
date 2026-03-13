import { pgTable, uuid, text, timestamp, index, real } from "drizzle-orm/pg-core";
import { boards } from "./boards.js";

export const boardColumns = pgTable(
  "board_columns",
  {
    id: uuid("id").primaryKey().defaultRandom(),
    boardId: uuid("board_id").notNull().references(() => boards.id, { onDelete: "cascade" }),
    name: text("name").notNull(),
    position: real("position").notNull().default(0), // fractional order
    createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
    updatedAt: timestamp("updated_at", { withTimezone: true }).notNull().defaultNow(),
  },
  (table) => ({
    boardIdx: index("board_columns_board_idx").on(table.boardId),
  }),
);
