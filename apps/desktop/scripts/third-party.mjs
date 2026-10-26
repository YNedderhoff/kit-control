import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import Handlebars from "handlebars";
import { parse } from "smol-toml";
import satisfies from "spdx-satisfies";

const exec = promisify(execFile);
const repositoryRoot = fileURLToPath(new URL("../../../", import.meta.url));
const cratesIo = "registry+https://github.com/rust-lang/crates.io-index";

async function runCommand(command, args, cwd) {
  try {
    return (await exec(command, args, { cwd, maxBuffer: 64 * 1024 * 1024 })).stdout;
  } catch (error) {
    throw new Error(`${command} ${args.join(" ")} failed:\n${error.stderr || error.message}`, { cause: error });
  }
}

async function readOptional(file) {
  try {
    return await readFile(file);
  } catch (error) {
    if (error.code === "ENOENT") return null;
    throw error;
  }
}

function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function normalize(text) {
  return text.replaceAll("\r\n", "\n");
}

function sourceInventory(report, lockfile) {
  if (!Array.isArray(report.licenses) || !report.licenses.length) {
    throw new Error("cargo-about returned no licenses");
  }
  const sources = new Map();
  for (const license of report.licenses.filter((item) => item.id === "MPL-2.0")) {
    for (const { crate } of license.used_by) {
      const { name, version, source } = crate;
      if (!/^[a-zA-Z0-9_-]+$/.test(name) || !/^[0-9][a-zA-Z0-9.+-]*$/.test(version)) {
        throw new Error(`Invalid crate identity: ${name}@${version}`);
      }
      if (source !== cratesIo) {
        throw new Error(`MPL source requires manual review: ${name}@${version} (${source})`);
      }
      const locked = lockfile.package.find((item) =>
        item.name === name && item.version === version && item.source === source);
      if (!locked || !/^[a-f0-9]{64}$/.test(locked.checksum)) {
        throw new Error(`Missing Cargo.lock checksum for ${name}@${version}`);
      }
      sources.set(`${name}@${version}`, {
        name, version, license: "MPL-2.0", modified: false,
        source: `https://crates.io/api/v1/crates/${name}/${version}/download`,
        archive: `mpl-sources/${name}-${version}.crate`,
        sha256: locked.checksum,
      });
    }
  }
  return [...sources.values()].sort((a, b) => compare(a.name, b.name) || compare(a.version, b.version));
}

// cargo-about can emit equal licenses in a different order on another host.
// Sort only presentation data; retain the selected licenses and upstream text.
function rustNotice(report, template) {
  const licenses = report.licenses.map((license) => ({
    ...license,
    text: normalize(license.text),
    used_by: [...license.used_by].sort((a, b) =>
      compare(a.crate.name, b.crate.name) || compare(a.crate.version, b.crate.version)),
  })).sort((a, b) => compare(a.id, b.id) || compare(a.text, b.text));
  return Handlebars.compile(template)({ ...report, licenses });
}

/** Generate or verify distribution assets without resolving new package versions. */
export async function maintainThirdParty({
  root = repositoryRoot,
  mode,
  run = runCommand,
  fetchArchive = fetch,
}) {
  if (!["update", "check"].includes(mode)) throw new Error("Use third-party.mjs update|check");
  const desktop = path.join(root, "apps/desktop");
  const manifest = JSON.parse(await readFile(path.join(desktop, "package.json"), "utf8"));
  const expectedVersion = manifest.kitcontrolMaintenance.cargoAboutVersion;
  const actualVersion = (await run("cargo", ["about", "--version"], root)).trim();
  if (actualVersion !== `cargo-about ${expectedVersion}`) {
    throw new Error(`Expected cargo-about ${expectedVersion}, got ${actualVersion}`);
  }

  const temporary = await mkdtemp(path.join(tmpdir(), "kitcontrol-notices-"));
  try {
    const report = JSON.parse(await run("cargo", [
      "about", "generate", "--workspace", "--locked", "--fail", "--format", "json",
    ], root));
    const lockfile = parse(await readFile(path.join(root, "Cargo.lock"), "utf8"));
    const sources = sourceInventory(report, lockfile);
    const template = await readFile(path.join(root, "about.hbs"), "utf8");
    const checker = path.join(desktop, "node_modules/license-checker-rseidelsohn/bin/license-checker-rseidelsohn.js");
    const accepted = parse(await readFile(path.join(root, "about.toml"), "utf8")).accepted;
    // The checker's --onlyAllow flag uses substring matching, which accepts
    // e.g. MIT AND an unapproved license. Evaluate the complete SPDX expression.
    const frontendLicenses = JSON.parse(await run(process.execPath,
      [checker, "--production", "--excludePrivatePackages", "--json"], desktop));
    const frontendAccepted = accepted.filter((license) => license !== "MPL-2.0");
    for (const [name, entry] of Object.entries(frontendLicenses)) {
      let approved = false;
      try {
        approved = typeof entry.licenses === "string" && satisfies(entry.licenses, frontendAccepted);
      } catch {
        // Missing, guessed or non-SPDX licenses need a maintainer decision.
      }
      if (!approved) throw new Error(`Frontend license requires review: ${name}: ${JSON.stringify(entry.licenses)}. npm MPL source distribution is not automated.`);
    }
    if (!Object.keys(frontendLicenses).length) throw new Error("Frontend license inventory is empty; run npm ci first");
    const frontend = path.join(temporary, "frontend.txt");
    await run(process.execPath, [checker, "--production", "--excludePrivatePackages",
      "--plainVertical", "--out", frontend], desktop);

    const outputs = new Map([
      ["THIRD_PARTY_RUST.md", Buffer.from(rustNotice(report, template))],
      ["THIRD_PARTY_FRONTEND.txt", Buffer.from(normalize(await readFile(frontend, "utf8")))],
      ["third-party/sources.json", Buffer.from(`${JSON.stringify(sources, null, 2)}\n`)],
    ]);
    const problems = [];
    for (const source of sources) {
      const relative = `third-party/${source.archive}`;
      let bytes = await readOptional(path.join(root, relative));
      if (!bytes && mode === "update") {
        const response = await fetchArchive(source.source, { signal: AbortSignal.timeout(60_000) });
        if (!response.ok) throw new Error(`Download ${source.source}: HTTP ${response.status}`);
        bytes = Buffer.from(await response.arrayBuffer());
      }
      if (!bytes) {
        problems.push(`Missing ${relative}`);
      } else if (createHash("sha256").update(bytes).digest("hex") !== source.sha256) {
        // A corrupt existing archive is evidence to investigate, not overwrite.
        problems.push(`Cargo.lock checksum mismatch: ${relative}`);
      } else {
        outputs.set(relative, bytes);
      }
    }
    if (problems.length) throw new Error(problems.join("\n"));

    const changed = [];
    for (const [relative, bytes] of outputs) {
      const existing = await readOptional(path.join(root, relative));
      if (!existing?.equals(bytes)) changed.push(relative);
    }
    if (mode === "check" && changed.length) {
      throw new Error(`Outdated distribution assets:\n${changed.join("\n")}\nRun npm --prefix apps/desktop run third-party:update`);
    }
    // Finish generators and validate every download before touching tracked files.
    for (const relative of mode === "update" ? changed : []) {
      const destination = path.join(root, relative);
      await mkdir(path.dirname(destination), { recursive: true });
      await writeFile(destination, outputs.get(relative));
    }
    return { changed, sources: sources.length };
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const result = await maintainThirdParty({ mode: process.argv[2] });
    console.log(`Third-party ${process.argv[2]} passed; ${result.sources} MPL sources, ${result.changed.length} changed files.`);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
