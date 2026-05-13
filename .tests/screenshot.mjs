#!/usr/bin/env node

import {chromium} from "playwright";
import path from "node:path";
import {pathToFileURL} from "node:url";
import fs from "node:fs/promises";

function printUsage() {
    console.error(`
Usage:
  browser-screenshot <input.html> <output.png> [width] [height] [deviceScaleFactor]

Example:
  browser-screenshot ./fixtures/button.html ./reference/button.png 800 600 1
`);
}

const [, , inputPath, outputPath, widthArg = "800", heightArg = "600", scaleArg = "1"] = process.argv;

if (!inputPath || !outputPath) {
    printUsage();
    process.exit(1);
}

const width = Number(widthArg);
const height = Number(heightArg);
const deviceScaleFactor = Number(scaleArg);

if (!Number.isFinite(width) || width <= 0) {
    console.error(`Invalid width: ${widthArg}`);
    process.exit(1);
}

if (!Number.isFinite(height) || height <= 0) {
    console.error(`Invalid height: ${heightArg}`);
    process.exit(1);
}

if (!Number.isFinite(deviceScaleFactor) || deviceScaleFactor <= 0) {
    console.error(`Invalid deviceScaleFactor: ${scaleArg}`);
    process.exit(1);
}

const fixture = path.resolve(inputPath);
const output = path.resolve(outputPath);

await fs.mkdir(path.dirname(output), {recursive: true});

const browser = await chromium.launch();

try {
    const page = await browser.newPage({
        viewport: {
            width,
            height,
        },
        deviceScaleFactor,
    });

    await page.goto(pathToFileURL(fixture).toString());

    await page.evaluate(() => document.fonts?.ready ?? Promise.resolve());

    await page.screenshot({
        path: output,
    });
} finally {
    await browser.close();
}
