CREATE TABLE "board_columns" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"board_id" uuid NOT NULL,
	"name" text NOT NULL,
	"position" real DEFAULT 0 NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "boards" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"company_id" uuid NOT NULL,
	"project_id" uuid,
	"name" text NOT NULL,
	"type" text DEFAULT 'kanban' NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "sprints" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"board_id" uuid NOT NULL,
	"name" text NOT NULL,
	"start_date" date,
	"end_date" date,
	"status" text DEFAULT 'planned' NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "issues" ADD COLUMN "board_id" uuid;--> statement-breakpoint
ALTER TABLE "issues" ADD COLUMN "board_column_id" uuid;--> statement-breakpoint
ALTER TABLE "issues" ADD COLUMN "sprint_id" uuid;--> statement-breakpoint
ALTER TABLE "issues" ADD COLUMN "position" real;--> statement-breakpoint
ALTER TABLE "board_columns" ADD CONSTRAINT "board_columns_board_id_boards_id_fk" FOREIGN KEY ("board_id") REFERENCES "public"."boards"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "boards" ADD CONSTRAINT "boards_company_id_companies_id_fk" FOREIGN KEY ("company_id") REFERENCES "public"."companies"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "boards" ADD CONSTRAINT "boards_project_id_projects_id_fk" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "sprints" ADD CONSTRAINT "sprints_board_id_boards_id_fk" FOREIGN KEY ("board_id") REFERENCES "public"."boards"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "board_columns_board_idx" ON "board_columns" USING btree ("board_id");--> statement-breakpoint
CREATE INDEX "boards_company_idx" ON "boards" USING btree ("company_id");--> statement-breakpoint
CREATE INDEX "boards_project_idx" ON "boards" USING btree ("project_id");--> statement-breakpoint
CREATE INDEX "sprints_board_idx" ON "sprints" USING btree ("board_id");--> statement-breakpoint
ALTER TABLE "issues" ADD CONSTRAINT "issues_board_id_boards_id_fk" FOREIGN KEY ("board_id") REFERENCES "public"."boards"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "issues" ADD CONSTRAINT "issues_board_column_id_board_columns_id_fk" FOREIGN KEY ("board_column_id") REFERENCES "public"."board_columns"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "issues" ADD CONSTRAINT "issues_sprint_id_sprints_id_fk" FOREIGN KEY ("sprint_id") REFERENCES "public"."sprints"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "issues_board_idx" ON "issues" USING btree ("board_id");--> statement-breakpoint
CREATE INDEX "issues_board_column_idx" ON "issues" USING btree ("board_column_id");--> statement-breakpoint
CREATE INDEX "issues_sprint_idx" ON "issues" USING btree ("sprint_id");