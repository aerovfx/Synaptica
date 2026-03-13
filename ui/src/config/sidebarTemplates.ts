/**
 * Company section nav items and which templates show them.
 * Top / Work / Projects / Agents are the same for all templates.
 */

export type UiTemplate = "company" | "school" | "hospital";

export interface CompanyNavItem {
  id: string;
  label: string;
  path: string;
}

/** All possible Company section items (id must match icon map in Sidebar). */
export const COMPANY_NAV_ITEMS: CompanyNavItem[] = [
  { id: "org", label: "Org", path: "/org" },
  { id: "dms", label: "Văn bản", path: "/dms" },
  { id: "spaces", label: "Spaces", path: "/spaces" },
  { id: "departments", label: "Tổ chuyên môn", path: "/departments" },
  { id: "classes", label: "Lớp học", path: "/classes" },
  { id: "social", label: "Mạng xã hội", path: "/social" },
  { id: "costs", label: "Costs", path: "/costs" },
  { id: "activity", label: "Activity", path: "/activity" },
  { id: "settings", label: "Settings", path: "/company/settings" },
];

/** Ids to show in Company section per template. */
const TEMPLATE_COMPANY_IDS: Record<UiTemplate, string[]> = {
  company: ["org", "costs", "activity", "settings"],
  school: [
    "org",
    "dms",
    "spaces",
    "departments",
    "classes",
    "social",
    "costs",
    "activity",
    "settings",
  ],
  hospital: ["org", "costs", "activity", "settings"],
};

export function getCompanyNavItems(template: UiTemplate): CompanyNavItem[] {
  const ids = TEMPLATE_COMPANY_IDS[template];
  return COMPANY_NAV_ITEMS.filter((item) => ids.includes(item.id));
}
