import { useEffect, useRef, useState } from "react";
import type { FormEvent } from "react";
import { core } from "./core";
import type {
  ApplyReport,
  ChangePlan,
  CoreClient,
  Demo,
  Document,
  Operation,
  Snapshot,
} from "./core";

const zoneNames: Record<string, string> = {
  "kick.head": "Kick · head",
  "snare.head": "Snare · head",
  "snare.rim": "Snare · rim",
  "tom1.head": "Tom 1 · head",
};

function instrumentName(id: string) {
  return id
    .replace("synth.", "")
    .replace(".01", " 01")
    .replace(/^\w/, (letter) => letter.toUpperCase());
}

function valueText(value: unknown): string {
  if (value === null) return "—";
  if (typeof value === "string") return value;
  return JSON.stringify(value);
}

function NumberField({
  label,
  value,
  min,
  max,
  disabled,
  onCommit,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  disabled: boolean;
  onCommit(value: number): Promise<boolean>;
}) {
  const [draft, setDraft] = useState(String(value));
  const cancelBlur = useRef(false);
  return (
    <label className="parameter">
      <span>
        {label}
        <small>
          {min} to {max}
        </small>
      </span>
      <input
        aria-label={label}
        type="number"
        min={min}
        max={max}
        step="1"
        value={draft}
        disabled={disabled}
        onChange={(event) => setDraft(event.target.value)}
        onBlur={(event) => {
          if (cancelBlur.current) {
            cancelBlur.current = false;
            return;
          }
          if (!event.target.validity.valid || draft.trim() === "") {
            setDraft(String(value));
            return;
          }
          const next = Number(draft);
          if (next !== value) {
            void onCommit(next).then((accepted) => {
              if (!accepted) setDraft(String(value));
            });
          }
        }}
        onKeyDown={(event) => {
          if (event.key === "Enter") event.currentTarget.blur();
          if (event.key === "Escape") {
            cancelBlur.current = true;
            setDraft(String(value));
            event.currentTarget.blur();
          }
        }}
      />
    </label>
  );
}

export function App({ client = core }: { client?: CoreClient }) {
  const [demo, setDemo] = useState<Demo | null>(null);
  const [document, setDocument] = useState<Document | null>(null);
  const [device, setDevice] = useState<Snapshot | null>(null);
  const [past, setPast] = useState<Document[]>([]);
  const [future, setFuture] = useState<Document[]>([]);
  const [selectedKit, setSelectedKit] = useState("");
  const [selectedPad, setSelectedPad] = useState("snare.head");
  const [name, setName] = useState("");
  const [workspacePath, setWorkspacePath] = useState("");
  const [savedDocument, setSavedDocument] = useState<Document | null>(null);
  const [savedPath, setSavedPath] = useState("");
  const [savedOrigin, setSavedOrigin] = useState<"loaded" | "saved">("saved");
  const [copyPadTarget, setCopyPadTarget] = useState("kick.head");
  const [copyKitTarget, setCopyKitTarget] = useState("2");
  const [plan, setPlan] = useState<ChangePlan | null>(null);
  const [report, setReport] = useState<ApplyReport | null>(null);
  const [fault, setFault] = useState<
    "none" | "disconnect" | "readback_mismatch"
  >("none");
  const [busy, setBusy] = useState("Starting simulator");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const running = useRef(false);

  useEffect(() => {
    let cancelled = false;
    client
      .demo()
      .then((result) => {
        if (cancelled) return;
        setDemo(result);
        setDocument(result.document);
        setDevice(result.snapshot);
        setSelectedKit(result.document.slots[0] ?? "");
        setBusy("");
      })
      .catch((reason: unknown) => {
        if (cancelled) return;
        setError(
          `Could not start the core. Open KitControl in the desktop app. ${reason instanceof Error ? reason.message : String(reason)}`,
        );
        setBusy("");
      });
    return () => {
      cancelled = true;
    };
  }, [client]);

  const kitId = document?.slots.includes(selectedKit)
    ? selectedKit
    : (document?.slots[0] ?? "");
  const kit = document?.kits[kitId];
  const padId = kit?.pads[selectedPad]
    ? selectedPad
    : (Object.keys(kit?.pads ?? {})[0] ?? "");
  const pad = kit?.pads[padId];
  const kitIndex = document?.slots.indexOf(kitId) ?? -1;

  useEffect(() => {
    setName(kit?.name ?? "");
  }, [kitId, kit?.name]);

  async function run(label: string, work: () => Promise<void>) {
    if (running.current) return false;
    running.current = true;
    setBusy(label);
    setError("");
    setNotice("");
    try {
      await work();
      return true;
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
      return false;
    } finally {
      running.current = false;
      setBusy("");
    }
  }

  function acceptEdit(next: Document) {
    if (document) setPast((items) => [...items, document]);
    setDocument(next);
    setFuture([]);
    setPlan(null);
    setReport(null);
  }

  function edit(operation: Operation) {
    if (!document) return Promise.resolve(false);
    return run("Updating workspace", async () => {
      acceptEdit(await client.edit(document, operation));
    });
  }

  function setParameter(parameter: string, value: number | string | null) {
    return edit({
      kind: "set_parameter",
      kit_id: kitId,
      pad: padId,
      parameter,
      value,
    });
  }

  function undo() {
    const previous = past.at(-1);
    if (!previous || !document || busy) return;
    setPast(past.slice(0, -1));
    setFuture([...future, document]);
    setDocument(previous);
    setPlan(null);
    setReport(null);
    setNotice("Undid the last edit.");
    setError("");
  }

  function redo() {
    const next = future.at(-1);
    if (!next || !document || busy) return;
    setFuture(future.slice(0, -1));
    setPast([...past, document]);
    setDocument(next);
    setPlan(null);
    setReport(null);
    setNotice("Redid the last edit.");
    setError("");
  }

  function moveKit(direction: -1 | 1) {
    if (!document) return;
    const order = [...document.slots];
    const destination = kitIndex + direction;
    if (destination < 0 || destination >= order.length) return;
    [order[kitIndex], order[destination]] = [
      order[destination],
      order[kitIndex],
    ];
    edit({ kind: "reorder_kits", order });
  }

  function rename(event: FormEvent) {
    event.preventDefault();
    if (name !== kit?.name) edit({ kind: "rename_kit", kit_id: kitId, name });
  }

  const disabled = Boolean(busy);
  const saveState = !document
    ? "Workspace unavailable"
    : document === savedDocument
      ? `${savedOrigin === "loaded" ? "Loaded" : "Saved"} · no unsaved changes`
      : document === demo?.document
        ? "New workspace · not saved"
        : "Unsaved changes";

  return (
    <div className="app-shell">
      <a className="skip-link" href="#editor">
        Skip to kit editor
      </a>
      <header className="topbar">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">
            k<span>c</span>
          </span>
          <div>
            KitControl<small>Offline kit editor</small>
          </div>
        </div>
        <div className="connection">
          <span
            className={`status-dot ${demo ? "" : "status-unavailable"}`}
            aria-hidden="true"
          />
          <strong>
            {demo
              ? "Simulator"
              : busy
                ? "Starting simulator"
                : "Core unavailable"}
          </strong>
          <span className="hardware-lock">Hardware locked</span>
        </div>
      </header>

      <div className="workspace-bar">
        <div className="workspace-title">
          <span className="eyebrow">KIT WORKSPACE</span>
          <strong aria-live="polite">{saveState}</strong>
          <small>
            {savedPath || "Save to a local folder to keep your edits."}
          </small>
        </div>
        <div className="workspace-controls">
          <label htmlFor="workspace-path">Workspace folder</label>
          <input
            id="workspace-path"
            value={workspacePath}
            onChange={(event) => setWorkspacePath(event.target.value)}
            placeholder="Absolute workspace folder path"
            spellCheck={false}
          />
          <button
            disabled={disabled || !document || !workspacePath.trim()}
            onClick={() => {
              void run("Loading workspace", async () => {
                const loaded = await client.load(workspacePath.trim());
                acceptEdit(loaded);
                setSavedDocument(loaded);
                setSavedPath(workspacePath.trim());
                setSavedOrigin("loaded");
                setSelectedKit(loaded.slots[0] ?? "");
                setNotice(
                  "Workspace loaded. Undo restores your previous workspace.",
                );
              });
            }}
          >
            Load
          </button>
          <button
            disabled={disabled || !document || !workspacePath.trim()}
            onClick={() => {
              if (!document) return;
              void run("Saving workspace", async () => {
                const saved = await client.save(workspacePath.trim(), document);
                setSavedDocument(document);
                setSavedPath(saved.path);
                setSavedOrigin("saved");
                setNotice(`Saved ${saved.files} files to ${saved.path}.`);
              });
            }}
          >
            Save workspace
          </button>
        </div>
      </div>

      <div className="feedback" aria-live="polite" aria-atomic="true">
        {busy ? (
          <span className="working">{busy}…</span>
        ) : notice ? (
          <span>{notice}</span>
        ) : (
          <span>
            Offline editing · synthetic parameters · hardware writes disabled
          </span>
        )}
      </div>
      {error && (
        <div className="error-banner" role="alert">
          <strong>Action could not complete.</strong> {error}
        </div>
      )}

      <main id="editor" className="editor-grid" aria-busy={disabled}>
        <aside className="kit-sidebar" aria-labelledby="kits-heading">
          <div className="section-heading">
            <h1 id="kits-heading">Your kits</h1>
            <span>{document?.slots.length ?? "—"} slots</span>
          </div>
          <p className="section-intro">Select a slot to edit its kit.</p>
          <div className="kit-list" aria-label="Kit slots">
            {document?.slots.map((id, index) => (
              <button
                key={`${id}-${index}`}
                className={`kit-card ${kitId === id ? "selected" : ""}`}
                aria-label={`Slot ${index + 1}: ${document.kits[id].name}`}
                aria-pressed={kitId === id}
                onClick={() => setSelectedKit(id)}
              >
                <span className="slot-number">
                  {String(index + 1).padStart(2, "0")}
                </span>
                <span>
                  <strong>{document.kits[id].name}</strong>
                  <small>
                    {Object.keys(document.kits[id].pads).length} zones
                  </small>
                </span>
                <span className="card-arrow" aria-hidden="true">
                  ↗
                </span>
              </button>
            ))}
          </div>
          <div className="reorder-controls">
            <button
              onClick={() => moveKit(-1)}
              disabled={disabled || kitIndex <= 0}
              aria-label="Move selected kit up"
            >
              ↑ Move up
            </button>
            <button
              onClick={() => moveKit(1)}
              disabled={
                disabled ||
                !document ||
                kitIndex < 0 ||
                kitIndex >= document.slots.length - 1
              }
              aria-label="Move selected kit down"
            >
              ↓ Move down
            </button>
          </div>
          <div className="sidebar-note">
            <span aria-hidden="true">◎</span>
            <strong>Simulator only</strong>
            <p>
              These kits use a synthetic profile. Real devices and manufacturer
              formats are not enabled.
            </p>
          </div>
        </aside>

        <section className="kit-workbench" aria-labelledby="kit-heading">
          <div className="workbench-heading">
            <div>
              <span className="eyebrow">
                KIT {String(kitIndex + 1).padStart(2, "0")}
              </span>
              <h2 id="kit-heading">
                {kit?.name ?? "Preparing your workspace"}
              </h2>
            </div>
            <div className="history">
              <button
                onClick={undo}
                disabled={disabled || !past.length}
                aria-label="Undo"
              >
                ↶ <span>Undo</span>
              </button>
              <button
                onClick={redo}
                disabled={disabled || !future.length}
                aria-label="Redo"
              >
                ↷ <span>Redo</span>
              </button>
            </div>
          </div>
          {kit && (
            <>
              <form className="rename-form" onSubmit={rename}>
                <label htmlFor="kit-name">Kit name</label>
                <input
                  id="kit-name"
                  maxLength={64}
                  value={name}
                  disabled={disabled}
                  onChange={(event) => setName(event.target.value)}
                />
                <button
                  disabled={disabled || name === kit.name || !name.trim()}
                  type="submit"
                >
                  Rename
                </button>
              </form>
              <div className="pad-stage">
                <div className="stage-caption">
                  <span>PAD MAP</span>
                  <span>Select a zone to shape its sound</span>
                </div>
                <div className="pad-map" role="group" aria-label="Pad zones">
                  {Object.keys(kit.pads).map((id) => (
                    <button
                      key={id}
                      className={`drum-pad pad-${id.replace(".", "-")} ${padId === id ? "active" : ""}`}
                      aria-label={`${zoneNames[id] ?? id} ${instrumentName(kit.pads[id].instrument)}`}
                      aria-pressed={padId === id}
                      onClick={() => setSelectedPad(id)}
                    >
                      <span className="pad-rings" aria-hidden="true" />
                      <span className="pad-label">{zoneNames[id] ?? id}</span>
                      <small>{instrumentName(kit.pads[id].instrument)}</small>
                    </button>
                  ))}
                </div>
                <div className="stage-bottom">
                  <span className="small-dot" />
                  <span>{zoneNames[padId] ?? padId} selected</span>
                  <span className="stage-profile">SYNTHETIC PROFILE</span>
                </div>
              </div>
              <div className="copy-tools">
                <div>
                  <h3>Copy a pad</h3>
                  <p>Copy this zone to another zone in the current kit.</p>
                  <div className="inline-controls">
                    <label htmlFor="copy-pad-target">Destination zone</label>
                    <select
                      id="copy-pad-target"
                      value={copyPadTarget}
                      onChange={(event) => setCopyPadTarget(event.target.value)}
                    >
                      {Object.keys(kit.pads).map((id) => (
                        <option key={id} value={id}>
                          {zoneNames[id] ?? id}
                        </option>
                      ))}
                    </select>
                    <button
                      disabled={
                        disabled ||
                        padId === copyPadTarget ||
                        !kit.pads[copyPadTarget]
                      }
                      onClick={() =>
                        edit({
                          kind: "copy_pad",
                          source_kit_id: kitId,
                          source_pad: padId,
                          target_kit_id: kitId,
                          target_pad: copyPadTarget,
                        })
                      }
                    >
                      Copy pad
                    </button>
                  </div>
                </div>
                <div>
                  <h3>Copy this kit</h3>
                  <p>Replace a destination slot. You can undo this edit.</p>
                  <div className="inline-controls">
                    <label htmlFor="copy-kit-target">Destination slot</label>
                    <select
                      id="copy-kit-target"
                      value={copyKitTarget}
                      onChange={(event) => setCopyKitTarget(event.target.value)}
                    >
                      {document?.slots.map((id, index) => (
                        <option key={`${id}-${index}`} value={index + 1}>
                          Slot {index + 1} · {document.kits[id].name}
                        </option>
                      ))}
                    </select>
                    <button
                      disabled={
                        disabled || Number(copyKitTarget) === kitIndex + 1
                      }
                      onClick={() =>
                        edit({
                          kind: "copy_kit",
                          source_kit_id: kitId,
                          target_slot: Number(copyKitTarget),
                          new_kit_id: `kit-${crypto.randomUUID()}`,
                        })
                      }
                    >
                      Copy kit
                    </button>
                  </div>
                </div>
              </div>
            </>
          )}
        </section>

        <aside
          key={`${kitId}:${padId}`}
          className="inspector"
          aria-labelledby="inspector-heading"
        >
          <div className="section-heading">
            <h2 id="inspector-heading">Sound detail</h2>
            <span className="detail-icon" aria-hidden="true">
              ☷
            </span>
          </div>
          <p className="section-intro">{zoneNames[padId] ?? "Select a zone"}</p>
          {pad && (
            <>
              <label className="instrument-select" htmlFor="instrument">
                Instrument
                <select
                  id="instrument"
                  aria-describedby="instrument-library"
                  disabled={disabled}
                  value={pad.instrument}
                  onChange={(event) =>
                    setParameter("instrument", event.target.value)
                  }
                >
                  {demo?.profile.instruments.map((id) => (
                    <option key={id} value={id}>
                      {instrumentName(id)}
                    </option>
                  ))}
                </select>
                <small id="instrument-library" aria-hidden="true">
                  Synthetic instrument library
                </small>
              </label>
              <div className="parameter-group">
                <span className="eyebrow">SHAPE</span>
                <NumberField
                  key={`tuning:${pad.tuning}`}
                  label="Tuning"
                  value={pad.tuning}
                  min={-100}
                  max={100}
                  disabled={disabled}
                  onCommit={(value) => setParameter("tuning", value)}
                />
                <NumberField
                  key={`muffling:${pad.muffling}`}
                  label="Muffling"
                  value={pad.muffling}
                  min={0}
                  max={10}
                  disabled={disabled}
                  onCommit={(value) => setParameter("muffling", value)}
                />
              </div>
              <div className="parameter-group">
                <span className="eyebrow">MIX</span>
                <NumberField
                  key={`volume:${pad.volume}`}
                  label="Volume"
                  value={pad.volume}
                  min={0}
                  max={100}
                  disabled={disabled}
                  onCommit={(value) => setParameter("volume", value)}
                />
                <NumberField
                  key={`pan:${pad.pan}`}
                  label="Pan"
                  value={pad.pan}
                  min={-30}
                  max={30}
                  disabled={disabled}
                  onCommit={(value) => setParameter("pan", value)}
                />
              </div>
              <div className="parameter-group">
                <span className="eyebrow">LAYER</span>
                <label className="instrument-select" htmlFor="sub-instrument">
                  Sub instrument
                  <select
                    id="sub-instrument"
                    disabled={disabled}
                    value={pad.sub_instrument ?? ""}
                    onChange={(event) =>
                      setParameter("sub_instrument", event.target.value || null)
                    }
                  >
                    <option value="">None</option>
                    {demo?.profile.instruments.map((id) => (
                      <option key={id} value={id}>
                        {instrumentName(id)}
                      </option>
                    ))}
                  </select>
                </label>
                <NumberField
                  key={`layer_level:${pad.layer_level}`}
                  label="Layer level"
                  value={pad.layer_level}
                  min={0}
                  max={100}
                  disabled={disabled}
                  onCommit={(value) => setParameter("layer_level", value)}
                />
              </div>
              <p className="input-hint">
                Press Enter or leave a value to commit. Every edit stays local
                until you plan and apply.
              </p>
            </>
          )}
        </aside>
      </main>

      <section className="review-panel" aria-labelledby="review-heading">
        <div className="review-header">
          <div>
            <span className="eyebrow">REVIEW & TRANSFER</span>
            <h2 id="review-heading">Review changes</h2>
            <p>Plan changes, apply to the simulator, and verify readback.</p>
          </div>
          <div className="review-actions">
            <button
              disabled={disabled || !document || !device}
              onClick={() => {
                if (!document || !device) return;
                void run("Planning changes", async () => {
                  setReport(null);
                  setPlan(await client.plan(device, document));
                });
              }}
            >
              1 · Plan changes
            </button>
            <button
              className="primary"
              disabled={disabled || !plan || !plan.changes.length || !device}
              onClick={() => {
                if (!plan || !device) return;
                void run("Applying and reading back", async () => {
                  const result = await client.apply(device, plan, fault);
                  setReport(result);
                  setDevice(result.snapshot);
                  setPlan(null);
                });
              }}
            >
              2 · Apply to simulator <span aria-hidden="true">↗</span>
            </button>
          </div>
        </div>

        {plan && (
          <div className="change-review">
            <h3>
              {plan.changes.length
                ? `${plan.changes.length} planned ${plan.changes.length === 1 ? "change" : "changes"}`
                : "Your workspace matches the simulator."}
            </h3>
            {plan.changes.length > 0 && (
              <div className="change-table-wrap">
                <table>
                  <thead>
                    <tr>
                      <th scope="col">Setting</th>
                      <th scope="col">Simulator</th>
                      <th scope="col">Workspace</th>
                    </tr>
                  </thead>
                  <tbody>
                    {plan.changes.map((change, index) => (
                      <tr key={`${change.path}-${index}`}>
                        <th scope="row">{change.path}</th>
                        <td>{valueText(change.before)}</td>
                        <td>{valueText(change.after)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}

        {report && (
          <div className={`apply-report report-${report.status}`} role="status">
            <strong>
              {report.status === "verified"
                ? "Readback verified"
                : `Apply ${report.status}`}
            </strong>
            <p>{report.message}</p>
            <span>
              {report.written} written · {report.verified} verified ·{" "}
              {report.total} total
            </span>
            {report.status !== "verified" && (
              <p>
                Your local edits remain available. Review a new plan against the
                latest simulator snapshot before trying again.
              </p>
            )}
          </div>
        )}
        <details className="simulator-options">
          <summary>Simulator options</summary>
          <label htmlFor="fault-mode">Apply scenario</label>
          <select
            id="fault-mode"
            value={fault}
            disabled={disabled}
            onChange={(event) => {
              setFault(event.target.value as typeof fault);
              setReport(null);
            }}
          >
            <option value="none">Normal readback</option>
            <option value="disconnect">Simulate a disconnect</option>
            <option value="readback_mismatch">
              Simulate a readback mismatch
            </option>
          </select>
          <p>Explore failure reports without connecting a device.</p>
        </details>
      </section>
      <footer>
        <span>
          KitControl <span className="footer-version">/ 0.1</span>
        </span>
        <span>
          {demo ? demo.profile.name : "Core connection required"} · Hardware
          writes disabled
        </span>
      </footer>
    </div>
  );
}
