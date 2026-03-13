const PROJECT_URL_KEY_DELIM_RE = /[^a-z0-9]+/g;
const PROJECT_URL_KEY_TRIM_RE = /^-+|-+$/g;

/** Vietnamese: đ/Đ → d; then NFD + strip combining marks. Same as agent slug (e.g. ưở → u). */
function removeVietnameseAccents(s: string): string {
  const dReplaced = s.replace(/\u0111/g, "d").replace(/\u0110/g, "D");
  const stripMarks = (t: string) => t.replace(/[\u0300-\u036f]/g, "");
  let out = stripMarks(dReplaced.normalize("NFD"));
  out = stripMarks(out.normalize("NFD"));
  return out;
}

export function normalizeProjectUrlKey(value: string | null | undefined): string | null {
  if (typeof value !== "string") return null;
  const noAccent = removeVietnameseAccents(value.trim());
  const normalized = noAccent
    .toLowerCase()
    .replace(PROJECT_URL_KEY_DELIM_RE, "-")
    .replace(PROJECT_URL_KEY_TRIM_RE, "");
  return normalized.length > 0 ? normalized : null;
}

/** Legacy: no Vietnamese accent removal (old slugs like c-m-tr-i). Used for fallback lookup. */
export function normalizeProjectUrlKeyLegacy(value: string | null | undefined): string | null {
  if (typeof value !== "string") return null;
  const normalized = value
    .trim()
    .toLowerCase()
    .replace(PROJECT_URL_KEY_DELIM_RE, "-")
    .replace(PROJECT_URL_KEY_TRIM_RE, "");
  return normalized.length > 0 ? normalized : null;
}

export function deriveProjectUrlKey(name: string | null | undefined, fallback?: string | null): string {
  return normalizeProjectUrlKey(name) ?? normalizeProjectUrlKey(fallback) ?? "project";
}

/** Legacy slug derivation (no accent removal). For resolving old URLs. */
export function deriveProjectUrlKeyLegacy(name: string | null | undefined, fallback?: string | null): string {
  return normalizeProjectUrlKeyLegacy(name) ?? normalizeProjectUrlKeyLegacy(fallback) ?? "project";
}
