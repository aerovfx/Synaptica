const AGENT_URL_KEY_DELIM_RE = /[^a-z0-9]+/g;
const AGENT_URL_KEY_TRIM_RE = /^-+|-+$/g;
const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

/** Vietnamese: đ/Đ → d; then NFD + strip combining marks. Some chars (e.g. ưở) decompose to ư+mark; ư needs a second NFD pass to become u. */
function removeVietnameseAccents(s: string): string {
  const dReplaced = s.replace(/\u0111/g, "d").replace(/\u0110/g, "D");
  const stripMarks = (t: string) => t.replace(/[\u0300-\u036f]/g, "");
  let out = stripMarks(dReplaced.normalize("NFD"));
  out = stripMarks(out.normalize("NFD"));
  return out;
}

export function isUuidLike(value: string | null | undefined): boolean {
  if (typeof value !== "string") return false;
  return UUID_RE.test(value.trim());
}

export function normalizeAgentUrlKey(value: string | null | undefined): string | null {
  if (typeof value !== "string") return null;
  const noAccent = removeVietnameseAccents(value.trim());
  const normalized = noAccent
    .toLowerCase()
    .replace(AGENT_URL_KEY_DELIM_RE, "-")
    .replace(AGENT_URL_KEY_TRIM_RE, "");
  return normalized.length > 0 ? normalized : null;
}

export function deriveAgentUrlKey(name: string | null | undefined, fallback?: string | null): string {
  return normalizeAgentUrlKey(name) ?? normalizeAgentUrlKey(fallback) ?? "agent";
}
