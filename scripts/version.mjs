import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const ROOT_DIR = process.cwd();
const PACKAGE_JSON_PATH = path.join(ROOT_DIR, "package.json");
const TAURI_CONFIG_PATH = path.join(ROOT_DIR, "src-tauri", "tauri.conf.json");
const CARGO_TOML_PATH = path.join(ROOT_DIR, "src-tauri", "Cargo.toml");
const VERSION_PATTERN = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

async function main() {
  const [command, ...rest] = process.argv.slice(2);

  if (command === "check") {
    await runCheck(parseTag(rest));
    return;
  }

  if (command === "set") {
    const version = rest[0];
    if (!version) {
      fail("Missing version. Usage: node scripts/version.mjs set <version>");
    }
    await runSet(version);
    return;
  }

  printUsage();
  process.exitCode = 1;
}

async function runCheck(expectedTag) {
  const versions = await readVersions();
  const uniqueVersions = new Set(Object.values(versions));

  if (uniqueVersions.size !== 1) {
    fail(
      [
        "Version mismatch detected.",
        `package.json: ${versions.packageJson}`,
        `src-tauri/tauri.conf.json: ${versions.tauriConfig}`,
        `src-tauri/Cargo.toml: ${versions.cargoToml}`
      ].join("\n")
    );
  }

  const [version] = uniqueVersions;
  if (expectedTag && expectedTag !== `v${version}`) {
    fail(`Git tag ${expectedTag} does not match manifest version v${version}.`);
  }

  console.log(`Version check passed: ${version}`);
}

async function runSet(version) {
  if (!VERSION_PATTERN.test(version)) {
    fail(`Invalid semver version: ${version}`);
  }

  const [packageJsonRaw, tauriConfigRaw, cargoTomlRaw] = await Promise.all([
    fs.readFile(PACKAGE_JSON_PATH, "utf8"),
    fs.readFile(TAURI_CONFIG_PATH, "utf8"),
    fs.readFile(CARGO_TOML_PATH, "utf8")
  ]);

  const packageJson = JSON.parse(packageJsonRaw);
  packageJson.version = version;

  const tauriConfig = JSON.parse(tauriConfigRaw);
  tauriConfig.version = version;

  const updatedCargoToml = cargoTomlRaw.replace(
    /^version = ".*"$/m,
    `version = "${version}"`
  );

  if (updatedCargoToml === cargoTomlRaw) {
    fail("Unable to update version in src-tauri/Cargo.toml.");
  }

  await Promise.all([
    fs.writeFile(PACKAGE_JSON_PATH, `${JSON.stringify(packageJson, null, 2)}\n`),
    fs.writeFile(TAURI_CONFIG_PATH, `${JSON.stringify(tauriConfig, null, 2)}\n`),
    fs.writeFile(CARGO_TOML_PATH, updatedCargoToml)
  ]);

  console.log(`Updated package.json, src-tauri/tauri.conf.json, and src-tauri/Cargo.toml to ${version}`);
}

async function readVersions() {
  const [packageJsonRaw, tauriConfigRaw, cargoTomlRaw] = await Promise.all([
    fs.readFile(PACKAGE_JSON_PATH, "utf8"),
    fs.readFile(TAURI_CONFIG_PATH, "utf8"),
    fs.readFile(CARGO_TOML_PATH, "utf8")
  ]);

  const cargoMatch = cargoTomlRaw.match(/^version = "(.*)"$/m);
  if (!cargoMatch) {
    fail("Unable to read version from src-tauri/Cargo.toml.");
  }

  return {
    packageJson: JSON.parse(packageJsonRaw).version,
    tauriConfig: JSON.parse(tauriConfigRaw).version,
    cargoToml: cargoMatch[1]
  };
}

function parseTag(args) {
  const tagIndex = args.findIndex((value) => value === "--tag");
  if (tagIndex === -1) {
    return null;
  }

  const tag = args[tagIndex + 1];
  if (!tag) {
    fail("Missing value for --tag.");
  }
  return tag;
}

function printUsage() {
  console.error("Usage:");
  console.error("  node scripts/version.mjs check [--tag v1.2.3]");
  console.error("  node scripts/version.mjs set <version>");
}

function fail(message) {
  throw new Error(message);
}

await main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
