import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

const repoRoot = process.cwd();
const androidManifestPath = resolve(
  repoRoot,
  "src-tauri",
  "gen",
  "android",
  "app",
  "src",
  "main",
  "AndroidManifest.xml"
);
const androidNetworkConfigPath = resolve(
  repoRoot,
  "src-tauri",
  "gen",
  "android",
  "app",
  "src",
  "main",
  "res",
  "xml",
  "network_security_config.xml"
);

const ANDROID_NETWORK_CONFIG = `<?xml version="1.0" encoding="utf-8"?>
<network-security-config>
  <base-config cleartextTrafficPermitted="true" />
</network-security-config>
`;

let changed = false;

if (existsSync(androidManifestPath)) {
  let manifest = readFileSync(androidManifestPath, "utf8");

  if (!manifest.includes('android.permission.INTERNET')) {
    manifest = manifest.replace(
      /<manifest\b([^>]*)>/,
      `<manifest$1>\n    <uses-permission android:name="android.permission.INTERNET" />`
    );
    changed = true;
  }

  if (!manifest.includes("android:usesCleartextTraffic=")) {
    manifest = manifest.replace(
      /<application\b/,
      '<application android:usesCleartextTraffic="true"'
    );
    changed = true;
  } else {
    manifest = manifest.replace(
      /android:usesCleartextTraffic="[^"]*"/,
      'android:usesCleartextTraffic="true"'
    );
  }

  if (!manifest.includes("android:networkSecurityConfig=")) {
    manifest = manifest.replace(
      /<application\b([^>]*)>/,
      '<application$1 android:networkSecurityConfig="@xml/network_security_config">'
    );
    changed = true;
  } else {
    manifest = manifest.replace(
      /android:networkSecurityConfig="[^"]*"/,
      'android:networkSecurityConfig="@xml/network_security_config"'
    );
  }

  writeFileSync(androidManifestPath, manifest);
  mkdirSync(dirname(androidNetworkConfigPath), { recursive: true });
  writeFileSync(androidNetworkConfigPath, ANDROID_NETWORK_CONFIG);
  console.log("Synced Android mobile network configuration.");
} else {
  console.log("Skipped Android mobile network sync because gen/android is not initialized.");
}

if (!changed && existsSync(androidManifestPath)) {
  console.log("Android manifest already contained the required network settings.");
}
