// SPDX-License-Identifier: Apache-2.0

import assert from "node:assert/strict";
import test from "node:test";

import { linkPrTask, listIssues, updateIssue } from "./github-admin.mjs";

function jsonResponse(payload) {
  return {
    ok: true,
    status: 200,
    async text() {
      return JSON.stringify(payload);
    },
    async json() {
      return payload;
    },
  };
}

async function withMockedFetch(handler, fn) {
  const originalFetch = global.fetch;
  const originalLog = console.log;
  const calls = [];
  const logs = [];

  process.env.GITHUB_TOKEN = "test-token";
  process.env.GITHUB_REPOSITORY = "acme/widgets";

  global.fetch = async (url, options = {}) => {
    const method = options.method || "GET";
    const rawBody = options.body;
    const body = rawBody ? JSON.parse(rawBody) : undefined;
    calls.push({ url, method, body });
    return handler({ url, method, body, calls });
  };
  console.log = (line) => {
    logs.push(line);
  };

  try {
    await fn({ calls, logs });
  } finally {
    global.fetch = originalFetch;
    console.log = originalLog;
    delete process.env.GITHUB_TOKEN;
    delete process.env.GITHUB_REPOSITORY;
  }
}

test("updateIssue supports milestone updates", async () => {
  await withMockedFetch(
    ({ url, method, body }) => {
      assert.equal(url, "https://api.github.com/repos/acme/widgets/issues/17");
      assert.equal(method, "PATCH");
      assert.deepEqual(body, { milestone: 5 });
      return jsonResponse({
        number: 17,
        state: "open",
        title: "Example",
        html_url: "https://github.com/acme/widgets/issues/17",
      });
    },
    async ({ logs }) => {
      await updateIssue({ number: "17", milestone: "5" });
      const output = JSON.parse(logs[0]);
      assert.equal(output.number, 17);
    },
  );
});

test("linkPrTask skips duplicate patch and comment when already linked", async () => {
  await withMockedFetch(
    ({ url, method }) => {
      if (url === "https://api.github.com/repos/acme/widgets/pulls/9") {
        assert.equal(method, "GET");
        return jsonResponse({
          number: 9,
          body: "Summary\n\nCloses #7",
          html_url: "https://github.com/acme/widgets/pull/9",
        });
      }
      if (url === "https://api.github.com/repos/acme/widgets/issues/7/comments?per_page=100") {
        assert.equal(method, "GET");
        return jsonResponse([
          {
            id: 11,
            body: "Linked pull request: [#9](https://github.com/acme/widgets/pull/9)",
          },
        ]);
      }
      assert.fail(`Unexpected request: ${method} ${url}`);
    },
    async ({ calls, logs }) => {
      await linkPrTask({ pr: "9", issue: "7", close: true });
      assert.equal(calls.length, 2);
      const output = JSON.parse(logs[0]);
      assert.equal(output.patchedBody, false);
      assert.equal(output.postedComment, false);
    },
  );
});

test("listIssues title filters are case-insensitive", async () => {
  await withMockedFetch(
    ({ url, method }) => {
      assert.equal(
        url,
        "https://api.github.com/repos/acme/widgets/issues?state=all&per_page=100&page=1",
      );
      assert.equal(method, "GET");
      return jsonResponse([
        { number: 1, state: "open", title: "[T2.7] Improve Sync", html_url: "x" },
      ]);
    },
    async ({ logs }) => {
      await listIssues({ state: "open", "title-contains": "improve sync" });
      const output = JSON.parse(logs[0]);
      assert.equal(output.length, 1);
      assert.equal(output[0].number, 1);
    },
  );
});
