#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0

import fs from "node:fs/promises";
import process from "node:process";
import { chromium } from "playwright";

function parseArgs(argv) {
  const options = {
    url: "",
    screenshot: "",
    dom: "",
    waitTexts: [],
    waitSelectors: [],
    forbidTexts: [],
    forbidSelectors: [],
    timeoutMs: 40_000,
    viewport: "1600,1400",
    chromePath: "/usr/bin/google-chrome",
    fullPage: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = argv[i + 1];

    if (arg === "--url") {
      options.url = next;
      i += 1;
    } else if (arg === "--screenshot") {
      options.screenshot = next;
      i += 1;
    } else if (arg === "--dom") {
      options.dom = next;
      i += 1;
    } else if (arg === "--wait-text") {
      options.waitTexts.push(next);
      i += 1;
    } else if (arg === "--wait-selector") {
      options.waitSelectors.push(next);
      i += 1;
    } else if (arg === "--forbid-text") {
      options.forbidTexts.push(next);
      i += 1;
    } else if (arg === "--forbid-selector") {
      options.forbidSelectors.push(next);
      i += 1;
    } else if (arg === "--timeout-ms") {
      options.timeoutMs = Number.parseInt(next, 10);
      i += 1;
    } else if (arg === "--viewport") {
      options.viewport = next;
      i += 1;
    } else if (arg === "--chrome-path") {
      options.chromePath = next;
      i += 1;
    } else if (arg === "--full-page") {
      options.fullPage = true;
    }
  }

  if (!options.url) throw new Error("Missing required --url");
  if (!options.screenshot) throw new Error("Missing required --screenshot");
  if (options.waitTexts.length === 0 && options.waitSelectors.length === 0) {
    throw new Error("Provide at least one --wait-text or --wait-selector");
  }

  return options;
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

async function waitForHydration(page, options) {
  await page.waitForFunction(
    ({ waitTexts, waitSelectors, forbidTexts, forbidSelectors }) => {
      const body = document.body;
      if (!body) return false;

      const text = (body.innerText ?? "").toLowerCase();
      const stylesheetReady = Array.from(
        document.querySelectorAll('link[rel="stylesheet"]')
      ).some((link) => link.href.includes("/assets/main-"));
      const backgroundImage = getComputedStyle(body).backgroundImage;
      const bodyStyled = backgroundImage && backgroundImage !== "none";
      const hasAllRequired = waitTexts.every((entry) =>
        text.includes(entry.toLowerCase())
      );
      const hasAllRequiredSelectors = waitSelectors.every((selector) =>
        document.querySelector(selector)
      );
      const hasForbidden = forbidTexts.some((entry) =>
        text.includes(entry.toLowerCase())
      );
      const hasForbiddenSelectors = forbidSelectors.some((selector) =>
        document.querySelector(selector)
      );

      return (
        document.readyState === "complete" &&
        stylesheetReady &&
        bodyStyled &&
        hasAllRequired &&
        hasAllRequiredSelectors &&
        !hasForbidden &&
        !hasForbiddenSelectors
      );
    },
    {
      waitTexts: options.waitTexts,
      waitSelectors: options.waitSelectors,
      forbidTexts: options.forbidTexts,
      forbidSelectors: options.forbidSelectors,
    },
    { timeout: options.timeoutMs }
  );
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const browser = await chromium.launch({
    executablePath: options.chromePath,
    headless: true,
    args: ["--no-sandbox", "--disable-gpu"],
  });

  try {
    const page = await browser.newPage({ viewport: parseViewport(options.viewport) });
    try {
      await page.goto(options.url, { waitUntil: "domcontentloaded" });
      await waitForHydration(page, options);

      const html = await page.content();
      await page.screenshot({
        path: options.screenshot,
        fullPage: options.fullPage,
      });

      if (options.dom) {
        await fs.writeFile(options.dom, html, "utf8");
      }

      process.stdout.write(`Verified and captured ${options.url}\n`);
    } catch (error) {
      const html = await page.content();
      await page.screenshot({
        path: options.screenshot,
        fullPage: options.fullPage,
      });

      if (options.dom) {
        await fs.writeFile(options.dom, html, "utf8");
      }

      throw error;
    }
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  process.stderr.write(`${error.message}\n`);
  process.exit(1);
});
