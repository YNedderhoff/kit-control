import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { App } from "./App";
import type { ApplyReport, ChangePlan, Demo, Document, Pad } from "./core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const pad: Pad = {
  instrument: "synth.snare.01",
  tuning: 0,
  muffling: 2,
  volume: 80,
  pan: 0,
  sub_instrument: null,
  layer_level: 0,
};
const document: Document = {
  format_version: 1,
  profile_id: "synthetic-v1",
  document_id: "test-workspace",
  kits: {
    a: {
      name: "Studio One",
      pads: {
        "kick.head": { ...pad, instrument: "synth.kick.01" },
        "snare.head": pad,
        "snare.rim": pad,
        "tom1.head": pad,
      },
      effects: {},
      opaque: {},
    },
    b: {
      name: "Soft Focus",
      pads: { "snare.head": pad },
      effects: {},
      opaque: {},
    },
    c: {
      name: "Night Drive",
      pads: { "snare.head": pad },
      effects: {},
      opaque: {},
    },
    d: {
      name: "Open Room",
      pads: { "snare.head": pad },
      effects: {},
      opaque: {},
    },
  },
  slots: ["a", "b", "c", "d"],
  setlists: [],
};
const edited: Document = {
  ...document,
  kits: {
    ...document.kits,
    a: {
      ...document.kits.a,
      pads: { ...document.kits.a.pads, "snare.head": { ...pad, tuning: 12 } },
    },
  },
};
const demo: Demo = {
  document,
  snapshot: {
    identity: {
      device_id: "sim-test",
      profile_id: "synthetic-v1",
      simulated: true,
    },
    document,
  },
  profile: {
    id: "synthetic-v1",
    name: "Synthetic Simulator",
    simulated: true,
    hardware_writes: false,
    instruments: [
      "synth.kick.01",
      "synth.snare.01",
      "synth.tom.01",
      "synth.cymbal.01",
    ],
  },
  hardware_status: "Synthetic workspace",
};
const plan: ChangePlan = {
  format_version: 1,
  expected_device: demo.snapshot.identity,
  baseline_hash: "baseline-test",
  desired_hash: "desired-test",
  baseline: document,
  desired: edited,
  changes: [{ path: "kits/a/pads/snare.head/tuning", before: 0, after: 12 }],
};
const verified: ApplyReport = {
  status: "verified",
  written: 1,
  verified: 1,
  total: 1,
  message: "Simulator readback matches the workspace.",
  snapshot: { identity: demo.snapshot.identity, document: edited },
  changes: [],
};

function reply(result: unknown) {
  return { api_version: 1, ok: true, result };
}
function pendingResponse() {
  let resolve!: (value: unknown) => void;
  const promise = new Promise<unknown>((finish) => {
    resolve = finish;
  });
  return { promise, resolve: (value: unknown) => resolve(value) };
}
async function openEditor() {
  render(<App />);
  await screen.findByRole("heading", { name: "Studio One" });
}
async function changeTuning() {
  const tuning = screen.getByRole("spinbutton", { name: "Tuning" });
  fireEvent.change(tuning, { target: { value: "12" } });
  fireEvent.blur(tuning);
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Undo" })).toBeEnabled(),
  );
}

beforeEach(() => {
  mockedInvoke.mockReset();
  mockedInvoke.mockResolvedValueOnce(reply(structuredClone(demo)));
});

describe("kit editing through the Rust dispatcher", () => {
  it("shows the simulator, four slots, and locks Apply until a plan exists", async () => {
    await openEditor();
    expect(screen.getByText("Offline workspace")).toBeVisible();
    expect(
      within(screen.getByLabelText("Kit slots")).getAllByRole("button"),
    ).toHaveLength(4);
    expect(
      screen.getByRole("button", { name: /Apply to simulator/ }),
    ).toBeDisabled();
    expect(mockedInvoke).toHaveBeenCalledWith("dispatch", {
      request: { action: "demo" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: /Kick · head Kick 01/ }),
    );
    expect(screen.getByRole("combobox", { name: "Instrument" })).toHaveValue(
      "synth.kick.01",
    );
  });

  it("sends parameter edits to the core and restores complete documents on undo and redo", async () => {
    mockedInvoke.mockResolvedValueOnce(reply(structuredClone(edited)));
    await openEditor();
    await changeTuning();
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: {
        action: "edit",
        document,
        operation: {
          kind: "set_parameter",
          kit_id: "a",
          pad: "snare.head",
          parameter: "tuning",
          value: 12,
        },
      },
    });
    fireEvent.click(screen.getByRole("button", { name: "Undo" }));
    expect(screen.getByRole("spinbutton", { name: "Tuning" })).toHaveValue(0);
    fireEvent.click(screen.getByRole("button", { name: "Redo" }));
    await waitFor(() =>
      expect(screen.getByRole("spinbutton", { name: "Tuning" })).toHaveValue(
        12,
      ),
    );
    expect(mockedInvoke).toHaveBeenCalledTimes(2);
  });

  it("commits the first numeric edit after delayed startup and restores committed values through history and loading", async () => {
    const startup = pendingResponse();
    const editResponse = pendingResponse();
    const loadResponse = pendingResponse();
    const loaded: Document = {
      ...document,
      kits: {
        ...document.kits,
        a: {
          ...document.kits.a,
          pads: {
            ...document.kits.a.pads,
            "snare.head": {
              instrument: "synth.snare.01",
              tuning: -25,
              muffling: 7,
              volume: 41,
              pan: -10,
              sub_instrument: null,
              layer_level: 65,
            },
          },
        },
      },
    };
    mockedInvoke.mockReset();
    mockedInvoke
      .mockReturnValueOnce(startup.promise)
      .mockReturnValueOnce(editResponse.promise)
      .mockReturnValueOnce(loadResponse.promise);
    render(<App />);
    expect(
      screen.queryByRole("spinbutton", { name: "Tuning" }),
    ).not.toBeInTheDocument();
    await act(async () => {
      startup.resolve(reply(structuredClone(demo)));
    });
    const tuning = screen.getByRole("spinbutton", { name: "Tuning" });
    fireEvent.change(tuning, { target: { value: "12" } });
    fireEvent.blur(tuning);
    expect(mockedInvoke).toHaveBeenNthCalledWith(2, "dispatch", {
      request: {
        action: "edit",
        document,
        operation: {
          kind: "set_parameter",
          kit_id: "a",
          pad: "snare.head",
          parameter: "tuning",
          value: 12,
        },
      },
    });
    expect(tuning).toHaveValue(12);
    expect(tuning).toBeDisabled();
    await act(async () => {
      editResponse.resolve(reply(structuredClone(edited)));
    });
    expect(screen.getByRole("spinbutton", { name: "Tuning" })).toHaveValue(12);
    expect(screen.getByText("Unsaved changes")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Undo" }));
    expect(screen.getByRole("spinbutton", { name: "Tuning" })).toHaveValue(0);
    fireEvent.click(screen.getByRole("button", { name: "Redo" }));
    expect(screen.getByRole("spinbutton", { name: "Tuning" })).toHaveValue(12);
    fireEvent.change(
      screen.getByRole("textbox", { name: "Workspace folder" }),
      { target: { value: "/tmp/lifecycle-kits" } },
    );
    fireEvent.click(screen.getByRole("button", { name: "Load" }));
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: { action: "load", path: "/tmp/lifecycle-kits" },
    });
    await act(async () => {
      loadResponse.resolve(reply(loaded));
    });
    expect(screen.getByRole("spinbutton", { name: "Tuning" })).toHaveValue(-25);
    expect(screen.getByRole("spinbutton", { name: "Muffling" })).toHaveValue(7);
    expect(screen.getByRole("spinbutton", { name: "Volume" })).toHaveValue(41);
    expect(screen.getByRole("spinbutton", { name: "Pan" })).toHaveValue(-10);
    expect(screen.getByRole("spinbutton", { name: "Layer level" })).toHaveValue(
      65,
    );
  });

  it("requires a reviewed plan, applies it, and displays the core readback report", async () => {
    mockedInvoke
      .mockResolvedValueOnce(reply(structuredClone(edited)))
      .mockResolvedValueOnce(reply(structuredClone(plan)))
      .mockResolvedValueOnce(reply(structuredClone(verified)));
    await openEditor();
    await changeTuning();
    fireEvent.click(screen.getByRole("button", { name: /Plan changes/ }));
    await screen.findByText("1 planned change");
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: { action: "plan", baseline: demo.snapshot, desired: edited },
    });
    expect(
      screen.getByRole("rowheader", { name: "kits/a/pads/snare.head/tuning" }),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: /Apply to simulator/ }));
    await screen.findByText("Readback verified");
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: {
        action: "simulate_apply",
        device: demo.snapshot,
        plan,
        fault: "none",
      },
    });
    expect(screen.getByText("1 written · 1 verified · 1 total")).toBeVisible();
    expect(
      screen.getByRole("button", { name: /Apply to simulator/ }),
    ).toBeDisabled();
  });

  it("invalidates the plan after undo so an old desired state cannot be applied", async () => {
    mockedInvoke
      .mockResolvedValueOnce(reply(structuredClone(edited)))
      .mockResolvedValueOnce(reply(structuredClone(plan)));
    await openEditor();
    await changeTuning();
    fireEvent.click(screen.getByRole("button", { name: /Plan changes/ }));
    await screen.findByText("1 planned change");
    fireEvent.click(screen.getByRole("button", { name: "Undo" }));
    expect(
      screen.getByRole("button", { name: /Apply to simulator/ }),
    ).toBeDisabled();
    expect(screen.queryByText("1 planned change")).not.toBeInTheDocument();
  });

  it("shows a backend error without changing the document or reporting success", async () => {
    mockedInvoke.mockResolvedValueOnce({
      api_version: 1,
      ok: false,
      error: { code: "validation", message: "Name is reserved" },
    });
    await openEditor();
    fireEvent.change(screen.getByRole("textbox", { name: "Kit name" }), {
      target: { value: "Reserved" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Rename" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Name is reserved (validation)",
    );
    expect(screen.getByRole("heading", { name: "Studio One" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Undo" })).toBeDisabled();
    expect(screen.queryByText("Readback verified")).not.toBeInTheDocument();
  });

  it("shows incomplete readback and requires a fresh plan for recovery", async () => {
    const failed: ApplyReport = {
      ...verified,
      status: "partial",
      written: 1,
      verified: 0,
      message: "Disconnected before readback.",
      snapshot: demo.snapshot,
    };
    mockedInvoke
      .mockResolvedValueOnce(reply(structuredClone(edited)))
      .mockResolvedValueOnce(reply(structuredClone(plan)))
      .mockResolvedValueOnce(reply(failed));
    await openEditor();
    await changeTuning();
    fireEvent.click(screen.getByRole("button", { name: /Plan changes/ }));
    await screen.findByText("1 planned change");
    fireEvent.change(screen.getByRole("combobox", { name: "Apply scenario" }), {
      target: { value: "disconnect" },
    });
    fireEvent.click(screen.getByRole("button", { name: /Apply to simulator/ }));
    await screen.findByText("Apply partial");
    expect(screen.getByText("1 written · 0 verified · 1 total")).toBeVisible();
    expect(screen.queryByText("Readback verified")).not.toBeInTheDocument();
    expect(screen.getByRole("spinbutton", { name: "Tuning" })).toHaveValue(12);
    expect(
      screen.getByRole("button", { name: /Apply to simulator/ }),
    ).toBeDisabled();
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: {
        action: "simulate_apply",
        device: demo.snapshot,
        plan,
        fault: "disconnect",
      },
    });
  });

  it("loads and saves the entered folder through the core and makes loading undoable", async () => {
    mockedInvoke
      .mockResolvedValueOnce(reply(structuredClone(edited)))
      .mockResolvedValueOnce(reply({ path: "/tmp/my-kits", files: 5 }));
    await openEditor();
    expect(screen.getByText("New workspace · not saved")).toBeInTheDocument();
    fireEvent.change(
      screen.getByRole("textbox", { name: "Workspace folder" }),
      { target: { value: "/tmp/my-kits" } },
    );
    fireEvent.click(screen.getByRole("button", { name: "Load" }));
    await screen.findByText(
      "Workspace loaded. Undo restores your previous workspace.",
    );
    expect(screen.getByText("Loaded · no unsaved changes")).toBeInTheDocument();
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: { action: "load", path: "/tmp/my-kits" },
    });
    await waitFor(() =>
      expect(screen.getByRole("spinbutton", { name: "Tuning" })).toHaveValue(
        12,
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "Save workspace" }));
    await screen.findByText("Saved 5 files to /tmp/my-kits.");
    expect(screen.getByText("Saved · no unsaved changes")).toBeInTheDocument();
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: { action: "save", path: "/tmp/my-kits", document: edited },
    });
    fireEvent.click(screen.getByRole("button", { name: "Undo" }));
    expect(screen.getByRole("spinbutton", { name: "Tuning" })).toHaveValue(0);
    expect(
      screen.queryByText("Saved · no unsaved changes"),
    ).not.toBeInTheDocument();
  });

  it("restores the committed numeric value when the core rejects an edit", async () => {
    mockedInvoke.mockResolvedValueOnce({
      api_version: 1,
      ok: false,
      error: { code: "transport", message: "Core unavailable" },
    });
    await openEditor();
    const tuning = screen.getByRole("spinbutton", { name: "Tuning" });
    fireEvent.change(tuning, { target: { value: "12" } });
    fireEvent.blur(tuning);
    await screen.findByRole("alert");
    await waitFor(() => expect(tuning).toHaveValue(0));
    expect(screen.getByRole("button", { name: "Undo" })).toBeDisabled();
  });

  it("marks edits unsaved and clears the flag only after the core saves them", async () => {
    mockedInvoke
      .mockResolvedValueOnce(reply(structuredClone(edited)))
      .mockResolvedValueOnce(reply({ path: "/tmp/my-kits", files: 5 }));
    await openEditor();
    expect(screen.getByText("New workspace · not saved")).toBeVisible();
    await changeTuning();
    expect(screen.getByText("Unsaved changes")).toBeVisible();
    fireEvent.change(
      screen.getByRole("textbox", { name: "Workspace folder" }),
      { target: { value: "/tmp/my-kits" } },
    );
    fireEvent.click(screen.getByRole("button", { name: "Save workspace" }));
    await screen.findByText("Saved · no unsaved changes");
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: { action: "save", path: "/tmp/my-kits", document: edited },
    });
  });

  it("rejects out-of-range entries and lets Escape cancel a draft", async () => {
    await openEditor();
    const tuning = screen.getByRole("spinbutton", { name: "Tuning" });
    fireEvent.change(tuning, { target: { value: "101" } });
    fireEvent.blur(tuning);
    expect(tuning).toHaveValue(0);
    fireEvent.focus(tuning);
    fireEvent.change(tuning, { target: { value: "25" } });
    fireEvent.keyDown(tuning, { key: "Escape" });
    expect(tuning).toHaveValue(0);
    expect(mockedInvoke).toHaveBeenCalledTimes(1);
  });

  it("submits a keyboard-accessible slot permutation and copy-pad destination", async () => {
    const reordered: Document = { ...document, slots: ["b", "a", "c", "d"] };
    mockedInvoke
      .mockResolvedValueOnce(reply(reordered))
      .mockResolvedValueOnce(reply(reordered));
    await openEditor();
    fireEvent.click(
      screen.getByRole("button", { name: "Move selected kit down" }),
    );
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledTimes(2));
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: {
        action: "edit",
        document,
        operation: { kind: "reorder_kits", order: ["b", "a", "c", "d"] },
      },
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Copy pad" })).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Copy pad" }));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledTimes(3));
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: {
        action: "edit",
        document: reordered,
        operation: {
          kind: "copy_pad",
          source_kit_id: "a",
          source_pad: "snare.head",
          target_kit_id: "a",
          target_pad: "kick.head",
        },
      },
    });
  });
});
