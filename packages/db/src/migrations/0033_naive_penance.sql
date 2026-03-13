CREATE TABLE "company_dms_documents" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"company_id" uuid NOT NULL,
	"title" text NOT NULL,
	"description" text,
	"type" text DEFAULT 'OTHER' NOT NULL,
	"file_size" integer DEFAULT 0 NOT NULL,
	"file_url" text DEFAULT '' NOT NULL,
	"uploaded_by_agent_id" uuid,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "company_dms_incoming" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"company_id" uuid NOT NULL,
	"title" text NOT NULL,
	"document_number" text,
	"type" text DEFAULT 'OTHER' NOT NULL,
	"status" text DEFAULT 'pending' NOT NULL,
	"priority" text DEFAULT 'normal' NOT NULL,
	"sender" text,
	"received_date" date NOT NULL,
	"deadline" date,
	"summary" text,
	"created_by_agent_id" uuid,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "company_dms_outgoing" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"company_id" uuid NOT NULL,
	"title" text NOT NULL,
	"document_number" text,
	"status" text DEFAULT 'draft' NOT NULL,
	"priority" text DEFAULT 'normal' NOT NULL,
	"recipient" text,
	"created_by_agent_id" uuid,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "company_dms_documents" ADD CONSTRAINT "company_dms_documents_company_id_companies_id_fk" FOREIGN KEY ("company_id") REFERENCES "public"."companies"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "company_dms_documents" ADD CONSTRAINT "company_dms_documents_uploaded_by_agent_id_agents_id_fk" FOREIGN KEY ("uploaded_by_agent_id") REFERENCES "public"."agents"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "company_dms_incoming" ADD CONSTRAINT "company_dms_incoming_company_id_companies_id_fk" FOREIGN KEY ("company_id") REFERENCES "public"."companies"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "company_dms_incoming" ADD CONSTRAINT "company_dms_incoming_created_by_agent_id_agents_id_fk" FOREIGN KEY ("created_by_agent_id") REFERENCES "public"."agents"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "company_dms_outgoing" ADD CONSTRAINT "company_dms_outgoing_company_id_companies_id_fk" FOREIGN KEY ("company_id") REFERENCES "public"."companies"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "company_dms_outgoing" ADD CONSTRAINT "company_dms_outgoing_created_by_agent_id_agents_id_fk" FOREIGN KEY ("created_by_agent_id") REFERENCES "public"."agents"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "company_dms_documents_company_idx" ON "company_dms_documents" USING btree ("company_id");--> statement-breakpoint
CREATE INDEX "company_dms_incoming_company_idx" ON "company_dms_incoming" USING btree ("company_id");--> statement-breakpoint
CREATE INDEX "company_dms_incoming_received_date_idx" ON "company_dms_incoming" USING btree ("received_date");--> statement-breakpoint
CREATE INDEX "company_dms_outgoing_company_idx" ON "company_dms_outgoing" USING btree ("company_id");