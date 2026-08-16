import assert from "node:assert/strict";
import test from "node:test";
import { macRelease } from "./public/release-download.js";

test("selects only the matching Apple silicon DMG", () => {
  const release = macRelease({
    tag_name: "v0.2.3",
    assets: [
      { name: "anastasia-0.2.3-linux-x86_64.tar.gz", browser_download_url: "linux" },
      { name: "anastasia-0.2.3-macos-arm64.dmg", browser_download_url: "mac" },
    ],
  });
  assert.deepEqual(release, { version: "0.2.3", url: "mac" });
  assert.equal(macRelease({ tag_name: "v0.2.4", assets: [] }), undefined);
});
