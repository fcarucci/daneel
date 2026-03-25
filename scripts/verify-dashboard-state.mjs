#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0

import fs from "node:fs/promises";
import process from "node:process";
import { chromium } from "playwright";

export function parseArgs(argv) {
  const options = {
    url: "",
    screenshot: "",
    dom: "",
    state: "",
    expectStale: null,
    expectSummaryStale: null,
    expectPill: "",
    waitTexts: ["Gateway status"],
    forbidTexts: [],
    timeoutMs: 40_000,
    viewport: "1600,1400",
    chromePath: "/usr/bin/google-chrome",
    fullPage: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = argv[index + 1];

    if (arg === "--url") {
      options.url = next;
      index += 1;
    } else if (arg === "--screenshot") {
      options.screenshot = next;
      index += 1;
    } else if (arg === "--dom") {
      options.dom = next;
      index += 1;
    } else if (arg === "--state") {
      options.state = next;
      index += 1;
    } else if (arg === "--expect-stale") {
      options.expectStale = parseBooleanFlag(next, arg);
      index += 1;
    } else if (arg === "--expect-summary-stale") {
      options.expectSummaryStale = parseBooleanFlag(next, arg);
      index += 1;
    } else if (arg === "--expect-pill") {
      options.expectPill = next;
      index += 1;
    } else if (arg === "--wait-text") {
      options.waitTexts.push(next);
      index += 1;
    } else if (arg === "--forbid-text") {
      options.forbidTexts.push(next);
      index += 1;
    } else if (arg === "--timeout-ms") {
      options.timeoutMs = Number.parseInt(next, 10);
      index += 1;
    } else if (arg === "--viewport") {
      options.viewport = next;
      index += 1;
    } else if (arg === "--chrome-path") {
      options.chromePath = next;
      index += 1;
    } else if (arg === "--full-page") {
      options.fullPage = true;
    }
  }

  if (!options.url) throw new Error("Missing required --url");
  if (!options.screenshot) throw new Error("Missing required --screenshot");
  if (!options.state) throw new Error("Missing required --state");

  return options;
}

function parseBooleanFlag(value, flagName) {
  if (value === "true") return true;
  if (value === "false") return false;
  throw new Error(`${flagName} must be true or false`);
}

function parseViewport(value) {
  const [width, height] = value
    .split(",")
    .map((entry) => Number.parseInt(entry, 10));
  if (!Number.isFinite(width) || !Number.isFinite(height)) {
    throw new Error(`Invalid --viewport value: ${value}`);
  }

  return { width, height };
}

export function matchesDashboardState(snapshot) {
  if (snapshot.readyState !== "complete") return false;
  if (!snapshot.stylesheetReady || !snapshot.bodyStyled) return false;
  if (!snapshot.hasAllRequired || snapshot.hasForbidden) return false;
  if (snapshot.graphView !== snapshot.state) return false;
  if (
    snapshot.expectStale !== null &&
    snapshot.graphStale !== String(snapshot.expectStale)
  ) {
    return false;
  }

  if (snapshot.expectSummaryStale !== null) {
    const hasExpectedSummaryStale = snapshot.summaryCards.some(
      (card) => card.stale === String(snapshot.expectSummaryStale)
    );
    if (!hasExpectedSummaryStale) return false;
  }

  if (
    snapshot.expectPill &&
    !snapshot.text.includes(snapshot.expectPill.toLowerCase())
  ) {
    return false;
  }

  return true;
}

globalThis.__daneelMatchesDashboardState = matchesDashboardState;

export async function verifyDashboardState(argv) {
  const options = parseArgs(argv);
  const browser = await chromium.launch({
    executablePath: options.chromePath,
    headless: true,
    args: ["--no-sandbox", "--disable-gpu"],
  });

  try {
    const page = await browser.newPage({ viewport: parseViewport(options.viewport) });
    await page.goto(options.url, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(
      ({ state, waitTexts, forbidTexts, expectStale, expectSummaryStale, expectPill }) => {
        const body = document.body;
        if (!body) return false;

        const text = (body.innerText ?? "").toLowerCase();
        const stylesheetReady = Array.from(
          document.querySelectorAll('link[rel="stylesheet"]')
        ).some((link) => link.href.includes("/assets/main-"));
        const backgroundImage = getComputedStyle(body).backgroundImage;
        const bodyStyled = backgroundImage && backgroundImage !== "none";
        const graphView = document.querySelector("[data-graph-view]");
        const hasAllRequired = waitTexts.every((entry) =>
          text.includes(entry.toLowerCase())
        );
        const hasForbidden = forbidTexts.some((entry) =>
          text.includes(entry.toLowerCase())
        );
        const summaryCards = Array.from(
          document.querySelectorAll("[data-summary-card]")
        ).map((card) => ({
          stale: card.getAttribute("data-summary-stale"),
        }));

        return globalThis.__daneelMatchesDashboardState({
          readyState: document.readyState,
          stylesheetReady,
          bodyStyled,
          graphView: graphView?.getAttribute("data-graph-view") ?? null,
          graphStale: graphView?.getAttribute("data-stale-view") ?? null,
          summaryCards,
          text,
          state,
          waitTexts,
          forbidTexts,
          expectStale,
          expectSummaryStale,
          expectPill,
          hasAllRequired,
          hasForbidden,
        });
      },
      {
        state: options.state,
        waitTexts: options.waitTexts,
        forbidTexts: options.forbidTexts,
        expectStale: options.expectStale,
        expectSummaryStale: options.expectSummaryStale,
        expectPill: options.expectPill,
      },
      { timeout: options.timeoutMs }
    );

    const html = await page.content();
    await page.screenshot({
      path: options.screenshot,
      fullPage: options.fullPage,
    });

    if (options.dom) {
      await fs.writeFile(options.dom, html, "utf8");
    }

    const result = {
      url: options.url,
      state: options.state,
      screenshot: options.screenshot,
      dom: options.dom,
      expectStale: options.expectStale,
      expectSummaryStale: options.expectSummaryStale,
      expectPill: options.expectPill,
      verified: true,
    };
    process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
    return result;
  } finally {
    await browser.close();
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  verifyDashboardState(process.argv.slice(2)).catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exit(1);
  });
}
