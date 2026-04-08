import fs from "node:fs/promises";
import path from "node:path";

const args = process.argv.slice(2);

function fail(message) {
  console.error(message);
  process.exit(1);
}

function parseArgs(argv) {
  const result = {};
  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];
    if (!token.startsWith("--")) {
      fail(`Unexpected argument: ${token}`);
    }
    const key = token.slice(2);
    const value = argv[i + 1];
    if (!value || value.startsWith("--")) {
      fail(`Missing value for --${key}`);
    }
    result[key] = value;
    i += 1;
  }
  return result;
}

async function ensureDir(dir) {
  await fs.mkdir(dir, { recursive: true });
}

async function listFilesRecursive(dir) {
  const entries = await fs.readdir(dir, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry) => {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        return listFilesRecursive(fullPath);
      }
      return fullPath;
    })
  );
  return files.flat();
}

async function copyWithName(source, outputDir, targetName) {
  const destination = path.join(outputDir, targetName);
  await fs.copyFile(source, destination);
}

function normalizedVersion(rawVersion) {
  return rawVersion.startsWith("v") ? rawVersion.slice(1) : rawVersion;
}

async function normalizeDesktop(options) {
  const inputRoot = options["input-root"];
  const outputDir = options["output-dir"];
  const platform = options.platform;
  const arch = options.arch;
  const version = normalizedVersion(options.version);

  if (!inputRoot || !outputDir || !platform || !arch || !version) {
    fail("desktop mode requires --input-root --output-dir --platform --arch --version");
  }

  const allowedExtensions = new Set([".exe", ".AppImage", ".deb", ".dmg"]);
  const files = (await listFilesRecursive(inputRoot))
    .filter((file) => allowedExtensions.has(path.extname(file)))
    .sort();

  if (files.length === 0) {
    fail("No desktop bundles were produced.");
  }

  await ensureDir(outputDir);

  for (const file of files) {
    const ext = path.extname(file);
    let targetName = "";
    if (ext === ".exe") {
      targetName = `BuYu_${version}_${platform}_${arch}_setup.exe`;
    } else if (ext === ".AppImage") {
      targetName = `BuYu_${version}_${platform}_${arch}.AppImage`;
    } else if (ext === ".deb") {
      targetName = `BuYu_${version}_${platform}_${arch}.deb`;
    } else if (ext === ".dmg") {
      targetName = `BuYu_${version}_${platform}_${arch}.dmg`;
    } else {
      fail(`Unsupported desktop bundle: ${file}`);
    }
    await copyWithName(file, outputDir, targetName);
  }
}

async function normalizeAndroid(options) {
  const inputRoot = options["input-root"];
  const outputDir = options["output-dir"];
  const platform = options.platform;
  const arch = options.arch;
  const version = normalizedVersion(options.version);

  if (!inputRoot || !outputDir || !platform || !arch || !version) {
    fail("android mode requires --input-root --output-dir --platform --arch --version");
  }

  const files = (await listFilesRecursive(inputRoot))
    .filter((file) => file.endsWith("-signed.apk"))
    .sort();

  if (files.length === 0) {
    fail("No signed APKs found.");
  }

  await ensureDir(outputDir);
  await copyWithName(files[0], outputDir, `BuYu_${version}_${platform}_${arch}.apk`);
}

async function normalizeIos(options) {
  const inputRoot = options["input-root"];
  const outputDir = options["output-dir"];
  const version = normalizedVersion(options.version);

  if (!inputRoot || !outputDir || !version) {
    fail("ios mode requires --input-root --output-dir --version");
  }

  const files = (await listFilesRecursive(inputRoot))
    .filter((file) => file.endsWith(".ipa"))
    .sort();

  if (files.length === 0) {
    fail("No iOS IPA found.");
  }

  await ensureDir(outputDir);
  await copyWithName(files[0], outputDir, `BuYu_${version}_ios.ipa`);
}

async function main() {
  const mode = args[0];
  const options = parseArgs(args.slice(1));

  if (mode === "desktop") {
    await normalizeDesktop(options);
    return;
  }

  if (mode === "android") {
    await normalizeAndroid(options);
    return;
  }

  if (mode === "ios") {
    await normalizeIos(options);
    return;
  }

  fail("Usage: node scripts/normalize-release-assets.mjs <desktop|android|ios> [options]");
}

await main();
