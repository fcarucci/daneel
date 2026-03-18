// SPDX-License-Identifier: Apache-2.0

import assert from "node:assert/strict";
import fs from "node:fs/promises";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  parseArgs,
  routeSlugFromUrl,
  uniquifyPathWithRouteAndTime,
  verifyRoute,
} from "./verify-live.mjs";

function listen(server) {
  return new Promise((resolve, reject) => {
    server.listen(0, "127.0.0.1", () => {
      resolve(server.address());
    });
    server.on("error", reject);
  });
}

function close(server) {
  return new Promise((resolve, reject) => {
    server.close((error) => {
      if (error) {
        reject(error);
        return;
      }
      resolve();
    });
  });
}

test("parseArgs enables waitConnected only when requested", () => {
  assert.equal(parseArgs([]).waitConnected, false);
  assert.equal(parseArgs(["--wait-connected"]).waitConnected, true);
});

test("route-based video naming stays stable and readable", () => {
  const namedPath = uniquifyPathWithRouteAndTime(
    "videos/output.mp4",
    "http://127.0.0.1:4127/agents"
  );

  assert.match(namedPath, /^videos\/agents_\d{8}_\d{6}\.mp4$/);
  assert.equal(routeSlugFromUrl("http://127.0.0.1:4127/"), "home");
});

test("verify-live waits until the pill becomes Connected", async () => {
  const screenshot = path.join(
    os.tmpdir(),
    `verify-live-${process.pid}-${Date.now()}.png`
  );
  const dom = path.join(
    os.tmpdir(),
    `verify-live-${process.pid}-${Date.now()}.html`
  );
  const server = http.createServer((request, response) => {
    if (request.url === "/assets/main-test.css") {
      response.writeHead(200, { "content-type": "text/css" });
      response.end("body { background-image: linear-gradient(#001122, #001122); }");
      return;
    }

    response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    response.end(`<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Daneel</title>
    <link rel="stylesheet" href="/assets/main-test.css" />
  </head>
  <body>
    <div data-live="false">Connecting</div>
    <p>Graph View</p>
    <p>Agent tiles</p>
    <p>Latest session: agent:test:main</p>
    <script>
      window.setTimeout(() => {
        const pill = document.querySelector("[data-live]");
        pill.dataset.live = "true";
        pill.textContent = "Connected";
      }, 250);
    </script>
  </body>
</html>`);
  });
  const address = await listen(server);

  try {
    const result = await verifyRoute([
      "--url",
      `http://${address.address}:${address.port}/agents`,
      "--screenshot",
      screenshot,
      "--dom",
      dom,
      "--wait-text",
      "Graph View",
      "--wait-text",
      "Agent tiles",
      "--forbid-text",
      "Gateway lookup failed",
      "--forbid-text",
      "Loading agents",
      "--min-latest-session-count",
      "1",
      "--wait-connected",
      "--timeout-ms",
      "5000",
    ]);

    assert.equal(result.verified, true);
    assert.equal(result.connectedRibbonPresent, true);
    assert.equal(result.latestSessionCount, 1);
  } finally {
    await close(server);
    await fs.rm(screenshot, { force: true });
    await fs.rm(dom, { force: true });
  }
});
