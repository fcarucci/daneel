// SPDX-License-Identifier: Apache-2.0

import assert from "node:assert/strict";
import test from "node:test";

import {
  matchesDashboardState,
  parseArgs,
} from "./verify-dashboard-state.mjs";

test("parseArgs requires a dashboard state", () => {
  assert.throws(
    () =>
      parseArgs([
        "--url",
        "http://127.0.0.1:4127/",
        "--screenshot",
        "/tmp/dashboard.png",
      ]),
    /Missing required --state/
  );
});

test("matchesDashboardState accepts a disconnected frozen dashboard snapshot", () => {
  const matches = matchesDashboardState({
    readyState: "complete",
    stylesheetReady: true,
    bodyStyled: true,
    graphView: "ready",
    graphStale: "true",
    summaryCards: [{ stale: "true" }, { stale: "true" }],
    text: "gateway status disconnected showing the last known graph snapshot",
    state: "ready",
    waitTexts: ["Gateway status"],
    forbidTexts: [],
    expectStale: true,
    expectSummaryStale: true,
    expectPill: "Disconnected",
    hasAllRequired: true,
    hasForbidden: false,
  });

  assert.equal(matches, true);
});

test("matchesDashboardState rejects mismatched graph state markers", () => {
  const matches = matchesDashboardState({
    readyState: "complete",
    stylesheetReady: true,
    bodyStyled: true,
    graphView: "loading",
    graphStale: "false",
    summaryCards: [{ stale: "false" }],
    text: "gateway status connected",
    state: "disconnected",
    waitTexts: ["Gateway status"],
    forbidTexts: [],
    expectStale: false,
    expectSummaryStale: false,
    expectPill: "Connected",
    hasAllRequired: true,
    hasForbidden: false,
  });

  assert.equal(matches, false);
});
