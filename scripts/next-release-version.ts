#!/usr/bin/env bun

export function nextReleaseVersion(
  latestTag?: string | null,
  cargoVersion?: string | null,
): string {
  const currentCargo = cargoVersion?.replace(/^v/i, "").trim() || "0.3.1";
  if (!latestTag || !latestTag.trim()) {
    return currentCargo;
  }
  const cleanTag = latestTag.replace(/^v/i, "").trim();

  const parse = (v: string) => {
    const match = v.match(/^(\d+)\.(\d+)\.(\d+)/);
    if (!match) return null;
    return [parseInt(match[1], 10), parseInt(match[2], 10), parseInt(match[3], 10)] as const;
  };

  const tagParsed = parse(cleanTag);
  const cargoParsed = parse(currentCargo);

  if (!tagParsed && !cargoParsed) return "0.3.1";
  if (!tagParsed) return currentCargo;
  if (!cargoParsed) return `${tagParsed[0]}.${tagParsed[1]}.${tagParsed[2] + 1}`;

  const [tMaj, tMin, tPatch] = tagParsed;
  const [cMaj, cMin, cPatch] = cargoParsed;

  if (
    cMaj > tMaj ||
    (cMaj === tMaj && cMin > tMin) ||
    (cMaj === tMaj && cMin === tMin && cPatch > tPatch)
  ) {
    return `${cMaj}.${cMin}.${cPatch}`;
  }

  return `${tMaj}.${tMin}.${tPatch + 1}`;
}

if (import.meta.main) {
  const latestTag = process.argv[2] ?? "";
  const cargoVersion = process.argv[3] ?? "";
  process.stdout.write(nextReleaseVersion(latestTag, cargoVersion));
}
