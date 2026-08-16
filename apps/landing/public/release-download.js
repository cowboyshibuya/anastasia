const RELEASE_API = "https://api.github.com/repos/cowboyshibuya/anastasia/releases/latest";

export function macRelease(release) {
  const version = String(release?.tag_name ?? "").replace(/^v/, "");
  const name = `anastasia-${version}-macos-arm64.dmg`;
  const asset = release?.assets?.find((candidate) => candidate.name === name);
  return asset && version
    ? { version, url: asset.browser_download_url }
    : undefined;
}

if (typeof document !== "undefined") {
  fetch(RELEASE_API, { headers: { Accept: "application/vnd.github+json" } })
    .then((response) => response.ok ? response.json() : undefined)
    .then(macRelease)
    .then((release) => {
      if (!release) return;
      for (const link of document.querySelectorAll("#nav-download, #hero-download, #closing-download")) {
        link.href = release.url;
        link.removeAttribute("aria-disabled");
        link.textContent = "Download for macOS";
      }
      document.querySelector("#ver").textContent = `v${release.version} · Apple silicon`;
    })
    .catch(() => {});
}
