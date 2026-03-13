import type { TranscriptEntry } from "../types";

export function parseOpenFangGatewayStdoutLine(line: string, ts: string): TranscriptEntry[] {
  return [{ kind: "stderr", ts, text: line }];
}
