import { useRef, useState } from "react";
import type { FormEvent } from "react";
import { td27Core } from "./td27";
import type {
  MidiPorts,
  Td27Changes,
  Td27Client,
  Td27Plan,
  Td27Report,
  Td27Snapshot,
  Td27Target,
} from "./td27";
import "./td27.css";

const padLabels = [
  ["kick", "Kick"],
  ["snare", "Snare"],
  ["tom1", "Tom 1"],
  ["tom2", "Tom 2"],
  ["tom3", "Tom 3"],
  ["hihat", "Hi-hat"],
  ["crash1", "Crash 1"],
  ["crash2", "Crash 2"],
  ["ride", "Ride"],
  ["aux1", "Aux 1"],
  ["aux2", "Aux 2"],
  ["aux3", "Aux 3"],
] as const;

function bytes(values: number[]) {
  return values
    .map((value) => value.toString(16).padStart(2, "0").toUpperCase())
    .join(" ");
}

function parameterLabel(key: string) {
  const kitLabels: Record<string, string> = {
    "kit.name": "Kit name",
    "kit.tempo": "Tempo",
    "kit.tempo_enabled": "Kit tempo enabled",
    "kit.brush_enabled": "Brush mode",
    "kit.xstick_enabled": "Cross-stick",
  };
  if (kitLabels[key]) return kitLabels[key];
  const pad = padLabels.find(([id]) => key === `pad.${id}.compressor_enabled`);
  return pad ? `${pad[1]} compressor` : key;
}

function displayValue(value: unknown) {
  if (typeof value === "boolean") return value ? "On" : "Off";
  return value === undefined ? "—" : String(value);
}

export function Td27Panel({ client = td27Core }: { client?: Td27Client }) {
  const [transport, setTransport] = useState<"native" | "emulator">("native");
  const [ports, setPorts] = useState<MidiPorts | null>(null);
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [deviceId, setDeviceId] = useState("16");
  const [firmwareConfirmed, setFirmwareConfirmed] = useState(false);
  const [kitNumber, setKitNumber] = useState("1");
  const [fault, setFault] = useState("none");
  const [snapshot, setSnapshot] = useState<Td27Snapshot | null>(null);
  const [emulatorState, setEmulatorState] = useState<Td27Snapshot | null>(null);
  const [changes, setChanges] = useState<Td27Changes>({});
  const [plan, setPlan] = useState<Td27Plan | null>(null);
  const [report, setReport] = useState<Td27Report | null>(null);
  const [readRequired, setReadRequired] = useState(false);
  const [busy, setBusy] = useState("");
  const [notice, setNotice] = useState("");
  const [error, setError] = useState("");
  const running = useRef(false);
  const isBusy = Boolean(busy);
  const isEmulator = transport === "emulator";
  const canRead = isEmulator || Boolean(input && output && firmwareConfirmed);

  function target(): Td27Target {
    if (transport === "native")
      return {
        transport,
        input,
        output,
        device_id: Number(deviceId),
        firmware: "2.00",
      };
    return {
      transport,
      ...(emulatorState ? { state: emulatorState } : {}),
      device_id: Number(deviceId),
      firmware: "2.00",
      fault,
    };
  }

  async function run(label: string, work: () => Promise<void>) {
    if (running.current) return;
    running.current = true;
    setBusy(label);
    setError("");
    setNotice("");
    try {
      await work();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      running.current = false;
      setBusy("");
    }
  }

  function invalidateConnection() {
    setSnapshot(null);
    setEmulatorState(null);
    setChanges({});
    setPlan(null);
    setReport(null);
    setReadRequired(false);
    setNotice("");
    setError("");
  }

  function draft(key: string, value: string | number | boolean) {
    setChanges((previous) => ({ ...previous, [key]: value }));
    setPlan(null);
    setReport(null);
    setNotice("");
  }

  function value(key: string) {
    return changes[key] ?? snapshot?.parameters[key];
  }

  function readKit(event: FormEvent) {
    event.preventDefault();
    if (!canRead) return;
    void run("Reading kit", async () => {
      setPlan(null);
      setReport(null);
      setReadRequired(true);
      const result = await client.read(target(), Number(kitNumber));
      setSnapshot(result);
      setChanges({});
      setReadRequired(false);
      if (isEmulator) setEmulatorState(result);
      setNotice(
        `Kit ${result.kit} read from ${isEmulator ? "the packet emulator" : "the selected TD-27"}.`,
      );
    });
  }

  function planChanges(event: FormEvent) {
    event.preventDefault();
    if (!snapshot || readRequired) return;
    void run("Planning changes", async () => {
      setPlan(null);
      setReport(null);
      setPlan(await client.plan(snapshot, changes));
    });
  }

  function applyPlan() {
    if (!plan || !plan.writes.length || readRequired) return;
    void run("Applying and reading back", async () => {
      setPlan(null);
      setReport(null);
      setReadRequired(true);
      const result = await client.apply(target(), plan);
      setReport(result);
      if (result.snapshot) {
        setSnapshot(result.snapshot);
        setReadRequired(false);
        if (isEmulator) setEmulatorState(result.snapshot);
      }
      if (result.status === "verified") setChanges({});
    });
  }

  function boolControl(key: string, label: string) {
    return (
      <label className="td-toggle" key={key}>
        <input
          type="checkbox"
          checked={value(key) === true}
          onChange={(event) => draft(key, event.target.checked)}
        />
        <span>{label}</span>
        <small aria-hidden="true">{value(key) === true ? "On" : "Off"}</small>
      </label>
    );
  }

  return (
    <div className="app-shell td-shell">
      <a className="skip-link" href="#td27-editor">
        Skip to TD-27 parameters
      </a>
      <header className="topbar">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">
            k<span>c</span>
          </span>
          <div>
            KitControl<small>TD-27 kit editor</small>
          </div>
        </div>
        <div className="connection">
          <span
            className={`status-dot ${snapshot && !readRequired ? "" : "status-unavailable"}`}
            aria-hidden="true"
          />
          <strong>{isEmulator ? "Packet emulator" : "Native MIDI"}</strong>
          <span className="hardware-lock">
            {isEmulator
              ? "No MIDI ports opened"
              : snapshot && !readRequired
                ? "Kit read"
                : "Read required"}
          </span>
        </div>
      </header>
      <main className="td-layout" id="td27-editor">
        <section className="td-connection-panel" aria-labelledby="td27-heading">
          <span className="eyebrow">DEVICE CONNECTION</span>
          <h1 id="td27-heading">TD-27</h1>
          <p className="td-description">
            Read a kit, edit supported parameters, then review and apply
            changes.
          </p>
          <form onSubmit={readKit}>
            <fieldset disabled={isBusy} className="td-fieldset">
              <legend>Transport</legend>
              <div className="td-transport">
                <label>
                  <input
                    type="radio"
                    name="transport"
                    checked={!isEmulator}
                    onChange={() => {
                      invalidateConnection();
                      setFirmwareConfirmed(false);
                      setTransport("native");
                    }}
                  />
                  Native MIDI
                </label>
                <label>
                  <input
                    type="radio"
                    name="transport"
                    checked={isEmulator}
                    onChange={() => {
                      invalidateConnection();
                      setTransport("emulator");
                    }}
                  />
                  Packet emulator
                </label>
              </div>
              {isEmulator ? (
                <>
                  <p className="td-emulator-note">
                    The emulator uses the same packet client with synthetic
                    parameter data. It does not open MIDI ports or connect to a
                    TD-27.
                  </p>
                  <label className="td-field">
                    Emulator scenario
                    <select
                      value={fault}
                      onChange={(event) => {
                        setFault(event.target.value);
                        setPlan(null);
                        setReport(null);
                      }}
                    >
                      <option value="none">Normal packets</option>
                      <option value="fragmented">Fragmented replies</option>
                      <option value="timeout">Reply timeout</option>
                      <option value="bad_checksum">
                        Invalid reply checksum
                      </option>
                      <option value="wrong_device">Wrong device reply</option>
                      <option value="wrong_model">Wrong model reply</option>
                      <option value="disconnect">
                        Disconnect during apply
                      </option>
                      <option value="readback_mismatch">
                        Readback mismatch
                      </option>
                    </select>
                  </label>
                  <p className="td-inline-note">
                    Packet faults also affect reads. Disconnect and readback
                    mismatch apply during writes.
                  </p>
                </>
              ) : (
                <>
                  <button
                    className="td-port-refresh"
                    type="button"
                    onClick={() => {
                      void run("Reading MIDI port names", async () => {
                        const result = await client.ports();
                        setPorts(result);
                        if (
                          (input && !result.inputs.includes(input)) ||
                          (output && !result.outputs.includes(output))
                        ) {
                          invalidateConnection();
                          setInput("");
                          setOutput("");
                        }
                        setNotice(
                          `${result.inputs.length} MIDI inputs and ${result.outputs.length} MIDI outputs found.`,
                        );
                      });
                    }}
                  >
                    Refresh MIDI ports
                  </button>
                  <label className="td-field">
                    MIDI input
                    <select
                      required
                      value={input}
                      onChange={(event) => {
                        invalidateConnection();
                        setFirmwareConfirmed(false);
                        setInput(event.target.value);
                      }}
                    >
                      <option value="">Select the TD-27 input</option>
                      {ports?.inputs.map((port, index) => (
                        <option key={`${port}-${index}`} value={port}>
                          {port}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label className="td-field">
                    MIDI output
                    <select
                      required
                      value={output}
                      onChange={(event) => {
                        invalidateConnection();
                        setFirmwareConfirmed(false);
                        setOutput(event.target.value);
                      }}
                    >
                      <option value="">Select the TD-27 output</option>
                      {ports?.outputs.map((port, index) => (
                        <option key={`${port}-${index}`} value={port}>
                          {port}
                        </option>
                      ))}
                    </select>
                  </label>
                  {ports && (!ports.inputs.length || !ports.outputs.length) && (
                    <p className="td-inline-note">
                      No complete MIDI port pair found. Connect the TD-27 and
                      refresh the ports.
                    </p>
                  )}
                  <div className="td-firmware">
                    <span>Documented firmware profile</span>
                    <strong>2.00</strong>
                    <p>
                      Firmware is declared manually. The identity response does
                      not verify this version.
                    </p>
                    <label>
                      <input
                        type="checkbox"
                        checked={firmwareConfirmed}
                        onChange={(event) => {
                          invalidateConnection();
                          setFirmwareConfirmed(event.target.checked);
                        }}
                      />
                      I checked that my TD-27 runs firmware 2.00.
                    </label>
                  </div>
                </>
              )}
              <div className="td-field-pair">
                <label className="td-field">
                  Device ID byte
                  <input
                    type="number"
                    required
                    min="16"
                    max="31"
                    step="1"
                    value={deviceId}
                    onChange={(event) => {
                      invalidateConnection();
                      setFirmwareConfirmed(false);
                      setDeviceId(event.target.value);
                    }}
                  />
                </label>
                <label className="td-field">
                  Kit slot
                  <input
                    type="number"
                    required
                    min="1"
                    max="100"
                    step="1"
                    value={kitNumber}
                    onChange={(event) => {
                      invalidateConnection();
                      setKitNumber(event.target.value);
                    }}
                  />
                </label>
              </div>
              <p className="td-inline-note">
                Device bytes 16–31 correspond to module display values 17–32.
              </p>
              <p className="td-inline-note">
                Changing the connection or kit clears its read and draft.
                Reading again replaces the draft.
              </p>
              <button
                className="primary td-read-button"
                type="submit"
                disabled={!canRead}
              >
                Read kit <span aria-hidden="true">↓</span>
              </button>
            </fieldset>
          </form>
          {snapshot && (
            <div className="td-identity">
              <span className="eyebrow">LAST IDENTITY RESPONSE</span>
              <dl>
                <div>
                  <dt>Device byte</dt>
                  <dd>{snapshot.identity.device_id}</dd>
                </div>
                <div>
                  <dt>Software revision bytes</dt>
                  <dd>{bytes(snapshot.identity.software_revision)}</dd>
                </div>
                <div>
                  <dt>Declared firmware</dt>
                  <dd>{snapshot.identity.firmware}</dd>
                </div>
              </dl>
            </div>
          )}
        </section>

        <section
          className="td-parameter-panel"
          aria-labelledby="td-parameters-heading"
        >
          <div className="td-editor-heading">
            <div>
              <span className="eyebrow">KIT PARAMETERS</span>
              <h2 id="td-parameters-heading">
                {snapshot ? `Kit ${snapshot.kit}` : "Read a kit to start"}
              </h2>
            </div>
            <span className="td-draft-state">
              {Object.keys(changes).length
                ? "Draft changes"
                : snapshot
                  ? "Read values"
                  : "No kit read"}
            </span>
          </div>
          <div className="td-feedback" aria-live="polite">
            {busy
              ? `${busy}…`
              : notice ||
                (readRequired
                  ? "Read the kit again before planning further changes."
                  : "Edits stay local until you apply a reviewed plan.")}
          </div>
          {error && (
            <div className="error-banner td-error" role="alert">
              <strong>Action could not complete.</strong> {error}
            </div>
          )}
          {snapshot ? (
            <form onSubmit={planChanges}>
              <fieldset
                className="td-fieldset"
                disabled={isBusy || readRequired}
              >
                <legend className="td-sr-only">Supported kit parameters</legend>
                <div className="td-kit-fields">
                  <label className="td-field">
                    Kit name
                    <input
                      value={
                        typeof value("kit.name") === "string"
                          ? String(value("kit.name"))
                          : ""
                      }
                      maxLength={12}
                      required
                      onChange={(event) =>
                        draft("kit.name", event.target.value)
                      }
                    />
                  </label>
                  <label className="td-field">
                    Tempo
                    <input
                      type="number"
                      required
                      min="20"
                      max="260"
                      step="1"
                      aria-describedby="td-tempo-range"
                      value={
                        typeof value("kit.tempo") === "number"
                          ? Number(value("kit.tempo"))
                          : ""
                      }
                      onChange={(event) =>
                        draft(
                          "kit.tempo",
                          Number.isNaN(event.target.valueAsNumber)
                            ? ""
                            : event.target.valueAsNumber,
                        )
                      }
                    />
                    <small id="td-tempo-range" aria-hidden="true">
                      20–260 BPM
                    </small>
                  </label>
                </div>
                <div className="td-kit-toggles">
                  {boolControl("kit.tempo_enabled", "Kit tempo enabled")}
                  {boolControl("kit.brush_enabled", "Brush mode")}
                  {boolControl("kit.xstick_enabled", "Cross-stick")}
                </div>
                <div className="td-section-label">
                  <h3>Pad compressors</h3>
                  <p>Enable or disable the compressor for each pad.</p>
                </div>
                <div className="td-pad-toggles">
                  {padLabels.map(([id, label]) =>
                    boolControl(
                      `pad.${id}.compressor_enabled`,
                      `${label} compressor`,
                    ),
                  )}
                </div>
                <p className="td-supported-note">
                  This profile edits kit name, tempo, kit switches, and pad
                  compressor switches. Other device settings remain outside this
                  editor.
                </p>
                <div className="td-plan-action">
                  <button type="submit" disabled={!Object.keys(changes).length}>
                    1 · Plan changes
                  </button>
                </div>
              </fieldset>
            </form>
          ) : (
            <div className="td-empty">
              <span aria-hidden="true">⇄</span>
              <h3>No kit loaded</h3>
              <p>
                {isEmulator
                  ? "Read a synthetic kit through the packet emulator."
                  : "Select both MIDI ports, confirm the firmware profile, and read a kit from the TD-27."}
              </p>
              <p>
                Supported: kit name, tempo, kit switches, and pad compressor
                switches.
              </p>
            </div>
          )}

          <section className="td-review" aria-labelledby="td-review-heading">
            <div className="td-review-heading">
              <div>
                <span className="eyebrow">PLAN · APPLY · READBACK</span>
                <h2 id="td-review-heading">Review device writes</h2>
              </div>
              <button
                className="primary"
                disabled={isBusy || !plan?.writes.length || readRequired}
                onClick={applyPlan}
              >
                2 · {isEmulator ? "Apply to emulator" : "Apply to TD-27"}
              </button>
            </div>
            {!plan && !report && (
              <p className="td-inline-note">
                A plan is required before Apply becomes available.
              </p>
            )}
            {plan && (
              <div className="td-plan-result">
                <h3>
                  {plan.writes.length
                    ? `${plan.writes.length} planned ${plan.writes.length === 1 ? "write" : "writes"}`
                    : "No writes needed. The draft matches the read values."}
                </h3>
                {plan.writes.length > 0 && (
                  <>
                    <p>
                      Kit {plan.baseline.kit} · device byte{" "}
                      {plan.baseline.identity.device_id} ·{" "}
                      {isEmulator
                        ? "packet emulator"
                        : "selected native MIDI ports"}
                    </p>
                    <div className="td-write-table">
                      <table>
                        <thead>
                          <tr>
                            <th scope="col">Parameter</th>
                            <th scope="col">Read value</th>
                            <th scope="col">Draft value</th>
                            <th scope="col">Address (hex)</th>
                            <th scope="col">Before (hex)</th>
                            <th scope="col">After (hex)</th>
                          </tr>
                        </thead>
                        <tbody>
                          {plan.writes.map((write, index) => (
                            <tr key={`${write.key}-${index}`}>
                              <th scope="row">{parameterLabel(write.key)}</th>
                              <td>
                                {displayValue(
                                  plan.baseline.parameters[write.key],
                                )}
                              </td>
                              <td>
                                {displayValue(
                                  changes[write.key] ??
                                    plan.baseline.parameters[write.key],
                                )}
                              </td>
                              <td>
                                <code>{bytes(write.address)}</code>
                              </td>
                              <td>
                                <code>{bytes(write.before)}</code>
                              </td>
                              <td>
                                <code>{bytes(write.after)}</code>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </>
                )}
              </div>
            )}
            {report && (
              <div
                className={`apply-report ${report.status === "verified" ? "report-verified" : "report-partial"}`}
                role="status"
              >
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
                    {readRequired
                      ? "Read the kit again to establish its current state. Your draft remains local."
                      : "Review a new plan against the returned snapshot before trying again. Your draft remains local."}
                  </p>
                )}
              </div>
            )}
          </section>
        </section>
      </main>
      <footer>
        <span>
          KitControl <span className="footer-version">/ 0.1</span>
        </span>
        <span>
          {isEmulator
            ? "Packet emulator · no hardware connection"
            : "Native MIDI · manually declared firmware 2.00"}
        </span>
      </footer>
    </div>
  );
}
