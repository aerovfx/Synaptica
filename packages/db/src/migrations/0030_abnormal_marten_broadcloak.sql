CREATE TABLE "company_departments" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"company_id" uuid NOT NULL,
	"space_id" uuid,
	"name" text NOT NULL,
	"leader_agent_id" uuid,
	"order" integer DEFAULT 0 NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "company_departments" ADD CONSTRAINT "company_departments_company_id_companies_id_fk" FOREIGN KEY ("company_id") REFERENCES "public"."companies"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "company_departments" ADD CONSTRAINT "company_departments_space_id_company_spaces_id_fk" FOREIGN KEY ("space_id") REFERENCES "public"."company_spaces"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "company_departments" ADD CONSTRAINT "company_departments_leader_agent_id_agents_id_fk" FOREIGN KEY ("leader_agent_id") REFERENCES "public"."agents"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "company_departments_company_idx" ON "company_departments" USING btree ("company_id");--> statement-breakpoint
CREATE INDEX "company_departments_space_idx" ON "company_departments" USING btree ("space_id");