import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { execFile } from "node:child_process";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { maintainThirdParty } from "./third-party.mjs";

const checksum = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

async function fixture(t) {
  const root = await mkdtemp(path.join(tmpdir(), "kitcontrol-maintenance-test-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const state = { version: "1.0.0", extra: false, failure: null, reverse: false };
  const write = async (relative, value) => {
    await mkdir(path.dirname(path.join(root, relative)), { recursive: true });
    await writeFile(path.join(root, relative), value);
  };
  await write("apps/desktop/package.json", '{"kitcontrolMaintenance":{"cargoAboutVersion":"0.9.2"}}');
  await write("apps/desktop/package-lock.json", "test-only npm lock\n");
  await write("about.toml", 'accepted = ["MIT", "MPL-2.0"]\n');
  await write("about.hbs", '{{#each licenses}}{{id}}:{{#each used_by}}{{crate.name}}@{{crate.version}};{{/each}}\n{{{text}}}\n{{/each}}');
  const setLock = async () => write("Cargo.lock", `version = 4
[[package]]
name = "sample"
version = "${state.version}"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "${checksum}"
[[package]]
name = "platform-only"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "${checksum}"
`);
  await setLock();
  const downloads = [];
  const run = async (command, args, cwd) => {
    if (command === "cargo" && args.join(" ") === "about --version") return "cargo-about 0.9.2\n";
    if (command === "cargo") {
      assert.deepEqual(args, ["about", "generate", "--workspace", "--locked", "--fail", "--format", "json"]);
      assert.equal(cwd, root);
      if (state.failure === "rust-license") throw new Error("Unaccepted license: test-only-license");
      const crates = [{ crate: { name: "sample", version: state.version,
        source: "registry+https://github.com/rust-lang/crates.io-index" } }];
      if (state.extra) crates.push({ crate: { name: "platform-only", version: "0.4.0",
        source: "registry+https://github.com/rust-lang/crates.io-index" } });
      if (state.reverse) crates.reverse();
      return JSON.stringify({ licenses: [{ id: "MPL-2.0", text: "Invented license text\r\n", used_by: crates }] });
    }
    assert.equal(command, process.execPath);
    assert.equal(cwd, path.join(root, "apps/desktop"));
    if (state.realChecker) {
      const checker = fileURLToPath(new URL("../node_modules/license-checker-rseidelsohn/bin/license-checker-rseidelsohn.js", import.meta.url));
      return (await promisify(execFile)(command, [checker, ...args.slice(1)], { cwd })).stdout;
    }
    if (args.at(-1) === "--json") return JSON.stringify({ "fixture@1.0.0": { licenses: state.failure === "frontend-license" ? "GPL-3.0-only" : "MIT" } });
    assert.deepEqual(args.slice(1, 5), ["--production", "--excludePrivatePackages", "--plainVertical", "--out"]);
    await writeFile(args.at(-1), "Invented frontend notice\r\n");
  };
  const fetchArchive = async (url) => {
    downloads.push(url);
    if (state.failure === "network") throw new Error("Connection refused");
    return { ok: state.failure !== "http", status: 503,
      arrayBuffer: async () => Buffer.from(state.failure === "checksum" ? "corrupted" : "abc") };
  };
  return { root, state, write, setLock, downloads,
    maintain: (mode) => maintainThirdParty({ root, mode, run, fetchArchive }) };
}

async function snapshot(root) {
  const files = {};
  for (const relative of (await readdir(root, { recursive: true })).sort()) {
    try {
      files[relative] = (await readFile(path.join(root, relative))).toString("base64");
    } catch (error) {
      if (error.code !== "EISDIR") throw error;
    }
  }
  return files;
}

test("given fresh assets when updating twice then output is stable and check is read-only", async (t) => {
  const f = await fixture(t);
  const cargoLock = await readFile(path.join(f.root, "Cargo.lock"));
  const npmLock = await readFile(path.join(f.root, "apps/desktop/package-lock.json"));
  await f.maintain("update");
  assert.equal(await readFile(path.join(f.root, "THIRD_PARTY_RUST.md"), "utf8"), "MPL-2.0:sample@1.0.0;\nInvented license text\n\n");
  assert.equal(await readFile(path.join(f.root, "THIRD_PARTY_FRONTEND.txt"), "utf8"), "Invented frontend notice\n");
  assert.deepEqual(JSON.parse(await readFile(path.join(f.root, "third-party/sources.json"))), [{
    name: "sample", version: "1.0.0", license: "MPL-2.0", modified: false,
    source: "https://crates.io/api/v1/crates/sample/1.0.0/download",
    archive: "mpl-sources/sample-1.0.0.crate", sha256: checksum,
  }]);
  const before = await snapshot(f.root);
  assert.deepEqual((await f.maintain("update")).changed, []);
  await f.maintain("check");
  assert.deepEqual(await snapshot(f.root), before);
  assert.deepEqual(await readFile(path.join(f.root, "Cargo.lock")), cargoLock);
  assert.deepEqual(await readFile(path.join(f.root, "apps/desktop/package-lock.json")), npmLock);
  assert.deepEqual(f.downloads, ["https://crates.io/api/v1/crates/sample/1.0.0/download"]);
});

test("given a version change and a new platform-only MPL crate then update discovers both", async (t) => {
  const f = await fixture(t);
  await f.maintain("update");
  f.state.version = "2.0.0";
  f.state.extra = true;
  await f.setLock();
  const before = await snapshot(f.root);
  await assert.rejects(f.maintain("check"), /Missing third-party\/mpl-sources\/platform-only/);
  assert.deepEqual(await snapshot(f.root), before);
  await f.maintain("update");
  assert.deepEqual(JSON.parse(await readFile(path.join(f.root, "third-party/sources.json"))).map((s) => [s.name, s.version]), [
    ["platform-only", "0.4.0"], ["sample", "2.0.0"],
  ]);
  const updated = await snapshot(f.root);
  f.state.reverse = true;
  await f.maintain("check");
  assert.deepEqual(await snapshot(f.root), updated);
});

test("given stale notices or inventory then check fails without changing files", async (t) => {
  for (const relative of ["THIRD_PARTY_RUST.md", "THIRD_PARTY_FRONTEND.txt", "third-party/sources.json"]) {
    await t.test(relative, async (t) => {
      const f = await fixture(t);
      await f.maintain("update");
      await f.write(relative, "stale\n");
      const before = await snapshot(f.root);
      await assert.rejects(f.maintain("check"), /Outdated distribution assets/);
      assert.deepEqual(await snapshot(f.root), before);
    });
  }
});

test("given a damaged archive then check and update fail without overwriting it", async (t) => {
  const f = await fixture(t);
  await f.maintain("update");
  await f.write("third-party/mpl-sources/sample-1.0.0.crate", "damaged");
  const before = await snapshot(f.root);
  await assert.rejects(f.maintain("check"), /Cargo.lock checksum mismatch/);
  await assert.rejects(f.maintain("update"), /Cargo.lock checksum mismatch/);
  assert.deepEqual(await snapshot(f.root), before);
});

test("given generator or download failures then update leaves the checkout unchanged", async (t) => {
  for (const [failure, message] of [
    ["rust-license", /Unaccepted license/], ["frontend-license", /Frontend license requires review/],
    ["network", /Connection refused/], ["http", /HTTP 503/], ["checksum", /Cargo.lock checksum mismatch/],
  ]) {
    await t.test(failure, async (t) => {
      const f = await fixture(t);
      f.state.failure = failure;
      const before = await snapshot(f.root);
      await assert.rejects(f.maintain("update"), message);
      assert.deepEqual(await snapshot(f.root), before);
    });
  }
});

test("given real npm license metadata then SPDX AND, OR and MPL obligations are checked", async (t) => {
  for (const [license, accepted] of [
    ["(MIT AND GPL-3.0-only)", false], ["(MIT OR GPL-3.0-only)", true],
    ["MPL-2.0", false], ["(MIT AND MPL-2.0)", false], ["(MIT OR MPL-2.0)", true],
  ]) {
    await t.test(license, async (t) => {
      const f = await fixture(t);
      f.state.realChecker = true;
      await f.write("apps/desktop/package.json", JSON.stringify({ name: "test-host", private: true, version: "1.0.0",
        dependencies: { "invented-package": "1.0.0" }, kitcontrolMaintenance: { cargoAboutVersion: "0.9.2" } }));
      await f.write("apps/desktop/package-lock.json", JSON.stringify({ name: "test-host", lockfileVersion: 3, packages: {
        "": { name: "test-host", version: "1.0.0", dependencies: { "invented-package": "1.0.0" } },
        "node_modules/invented-package": { version: "1.0.0", license },
      } }));
      await f.write("apps/desktop/node_modules/invented-package/package.json", JSON.stringify({ name: "invented-package", version: "1.0.0", license }));
      await f.write("apps/desktop/node_modules/invented-package/LICENSE", "Test-only invented license fixture\n");
      if (accepted) {
        await f.maintain("update");
      } else {
        const before = await snapshot(f.root);
        await assert.rejects(f.maintain("update"), /Frontend license requires review/);
        assert.deepEqual(await snapshot(f.root), before);
      }
    });
  }
});

test("given a missing locked checksum then update refuses the source", async (t) => {
  const f = await fixture(t);
  await f.write("Cargo.lock", 'version = 4\npackage = []\n');
  await assert.rejects(f.maintain("update"), /Missing Cargo.lock checksum/);
  assert.deepEqual(f.downloads, []);
});

test("given another cargo-about version then maintenance fails before generating assets", async (t) => {
  const f = await fixture(t);
  await assert.rejects(maintainThirdParty({ root: f.root, mode: "check", run: async () => "cargo-about 99.0.0" }), /Expected cargo-about 0.9.2/);
});
