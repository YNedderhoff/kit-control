import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { KitControl } from "./KitControl";
import { Td27Panel } from "./Td27Panel";
import type { Td27Plan, Td27Report, Td27Snapshot } from "./td27";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);
const snapshot: Td27Snapshot = {
  format_version: 1,
  simulated: true,
  endpoint: null,
  identity: {
    device_id: 16,
    software_revision: [0, 0, 0, 2],
    firmware: "2.00",
  },
  kit: 1,
  blocks: [
    // Handwritten interface expectations: Kit Common and twelve Pad blocks.
    { address: [4, 0, 0, 0], data: [
      83, 116, 117, 100, 105, 111, 32, 79, 110, 101, 32, 32,
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      7, 8, 0, 0,
    ] },
    { address: [4, 1, 64, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 65, 0], data: [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 66, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 67, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 68, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 69, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 70, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 71, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 72, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 73, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 74, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { address: [4, 1, 75, 0], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
  ],
  parameters: {
    "kit.name": "Studio One",
    "kit.tempo": 120,
    "kit.tempo_enabled": false,
    "kit.brush_enabled": false,
    "kit.xstick_enabled": true,
    "pad.kick.compressor_enabled": false,
    "pad.snare.compressor_enabled": true,
    "pad.tom1.compressor_enabled": false,
    "pad.tom2.compressor_enabled": false,
    "pad.tom3.compressor_enabled": false,
    "pad.hihat.compressor_enabled": false,
    "pad.crash1.compressor_enabled": false,
    "pad.crash2.compressor_enabled": false,
    "pad.ride.compressor_enabled": false,
    "pad.aux1.compressor_enabled": false,
    "pad.aux2.compressor_enabled": false,
    "pad.aux3.compressor_enabled": false,
  },
};
const changedSnapshot: Td27Snapshot = {
  ...snapshot,
  blocks: [
    { address: [4, 0, 0, 0], data: [
      83, 116, 117, 100, 105, 111, 32, 79, 110, 101, 32, 32,
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      7, 13, 0, 0,
    ] },
    ...snapshot.blocks.slice(1),
  ],
  parameters: { ...snapshot.parameters, "kit.tempo": 125 },
};
const plan: Td27Plan = {
  format_version: 1,
  baseline: snapshot,
  baseline_hash: "independent-test-hash",
  changes: { "kit.tempo": 125 },
  writes: [
    {
      key: "kit.tempo",
      address: [4, 0, 0, 46],
      before: [0, 0, 7, 8],
      after: [0, 0, 7, 13],
    },
  ],
};
const report: Td27Report = {
  status: "verified",
  written: 1,
  verified: 1,
  total: 1,
  message: "Readback matches the planned write.",
  snapshot: changedSnapshot,
};
const emulatorTarget = {
  transport: "emulator",
  device_id: 16,
  firmware: "2.00",
  fault: "none",
};

function reply(result: unknown) {
  return { api_version: 1, ok: true, result };
}
async function readEmulator() {
  render(<Td27Panel />);
  fireEvent.click(screen.getByRole("radio", { name: "Sandbox (TD-27 emulator)" }));
  fireEvent.click(screen.getByRole("button", { name: "Read kit" }));
  await screen.findByRole("textbox", { name: "Kit name" });
  await waitFor(() =>
    expect(screen.getByRole("spinbutton", { name: "Tempo" })).toBeEnabled(),
  );
}
async function planTempo() {
  fireEvent.change(screen.getByRole("spinbutton", { name: "Tempo" }), {
    target: { value: "125" },
  });
  fireEvent.click(screen.getByRole("button", { name: /Plan changes/ }));
  await screen.findByText("1 planned write");
}

beforeEach(() => mockedInvoke.mockReset());

describe("TD-27 device integration", () => {
  it("opens the native editor by default and requires explicit ports and firmware declaration", async () => {
    const nativeSnapshot: Td27Snapshot = {
      ...snapshot,
      simulated: false,
      endpoint: { input: "TD-27 input", output: "TD-27 output" },
    };
    const nativePlan: Td27Plan = { ...plan, baseline: nativeSnapshot };
    const nativeReport: Td27Report = {
      ...report,
      snapshot: {
        ...changedSnapshot,
        simulated: false,
        endpoint: nativeSnapshot.endpoint,
      },
    };
    mockedInvoke
      .mockResolvedValueOnce(
        reply({ inputs: ["TD-27 input"], outputs: ["TD-27 output"] }),
      )
      .mockResolvedValueOnce(reply(nativeSnapshot))
      .mockResolvedValueOnce(reply(nativePlan))
      .mockResolvedValueOnce(reply(nativeReport));
    render(<KitControl />);
    expect(
      screen.getByRole("button", { name: "TD-27 device" }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("radio", { name: "Native MIDI" })).toBeChecked();
    expect(screen.getByRole("button", { name: "Read kit" })).toBeDisabled();
    expect(mockedInvoke).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Refresh MIDI ports" }));
    await screen.findByRole("option", { name: "TD-27 input" });
    await waitFor(() =>
      expect(
        screen.getByRole("combobox", { name: "MIDI input" }),
      ).toBeEnabled(),
    );
    fireEvent.change(screen.getByRole("combobox", { name: "MIDI input" }), {
      target: { value: "TD-27 input" },
    });
    fireEvent.change(screen.getByRole("combobox", { name: "MIDI output" }), {
      target: { value: "TD-27 output" },
    });
    expect(screen.getByRole("button", { name: "Read kit" })).toBeDisabled();
    fireEvent.click(
      screen.getByRole("checkbox", {
        name: "I checked that my TD-27 runs firmware 2.00.",
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Read kit" }));
    await screen.findByRole("textbox", { name: "Kit name" });
    const target = {
      transport: "native",
      input: "TD-27 input",
      output: "TD-27 output",
      device_id: 16,
      firmware: "2.00",
    };
    expect(mockedInvoke).toHaveBeenNthCalledWith(1, "dispatch", {
      request: { action: "midi_ports" },
    });
    expect(mockedInvoke).toHaveBeenNthCalledWith(2, "dispatch", {
      request: { action: "td27_read", target, kit: 1 },
    });
    expect(
      screen.getByText(
        "Firmware is declared manually. The identity response does not verify this version.",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: /Apply to TD-27/ }),
    ).toBeDisabled();
    await planTempo();
    expect(screen.getByRole("cell", { name: "04 00 00 2E" })).toBeVisible();
    expect(screen.getByRole("cell", { name: "00 00 07 08" })).toBeVisible();
    expect(screen.getByRole("cell", { name: "00 00 07 0D" })).toBeVisible();
    expect(mockedInvoke).toHaveBeenNthCalledWith(3, "dispatch", {
      request: {
        action: "td27_plan",
        baseline: nativeSnapshot,
        changes: { "kit.tempo": 125 },
      },
    });
    fireEvent.click(screen.getByRole("button", { name: /Apply to TD-27/ }));
    await screen.findByText("Readback verified");
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: { action: "td27_apply", target, plan: nativePlan },
    });
    expect(screen.getByText("1 written · 1 verified · 1 total")).toBeVisible();
  });

  it("runs the packet emulator without listing ports and reuses its returned state", async () => {
    mockedInvoke
      .mockResolvedValueOnce(reply(snapshot))
      .mockResolvedValueOnce(reply(plan))
      .mockResolvedValueOnce(reply(report))
      .mockResolvedValueOnce(reply(changedSnapshot));
    await readEmulator();
    expect(screen.getByText(/It does not open MIDI ports/)).toBeVisible();
    expect(
      screen.queryByRole("combobox", { name: "MIDI input" }),
    ).not.toBeInTheDocument();
    expect(mockedInvoke).toHaveBeenNthCalledWith(1, "dispatch", {
      request: { action: "td27_read", target: emulatorTarget, kit: 1 },
    });
    await planTempo();
    fireEvent.click(screen.getByRole("button", { name: /Apply in sandbox/ }));
    await screen.findByText("Readback verified");
    expect(mockedInvoke).toHaveBeenNthCalledWith(3, "dispatch", {
      request: {
        action: "td27_apply",
        target: { ...emulatorTarget, state: snapshot },
        plan,
      },
    });
    fireEvent.click(screen.getByRole("button", { name: "Read kit" }));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledTimes(4));
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: {
        action: "td27_read",
        target: { ...emulatorTarget, state: changedSnapshot },
        kit: 1,
      },
    });
  });

  it("invalidates plans on draft and connection changes and clears emulator state for a new kit", async () => {
    mockedInvoke
      .mockResolvedValueOnce(reply(snapshot))
      .mockResolvedValueOnce(reply(plan))
      .mockResolvedValueOnce(reply(plan))
      .mockResolvedValueOnce(reply({ ...snapshot, kit: 2 }));
    await readEmulator();
    await planTempo();
    fireEvent.change(screen.getByRole("spinbutton", { name: "Tempo" }), {
      target: { value: "130" },
    });
    expect(screen.queryByText("1 planned write")).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Apply in sandbox/ }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: /Plan changes/ }));
    await screen.findByText("1 planned write");
    fireEvent.change(screen.getByRole("spinbutton", { name: "Kit slot" }), {
      target: { value: "2" },
    });
    expect(
      screen.queryByRole("textbox", { name: "Kit name" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Apply in sandbox/ }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Read kit" }));
    await screen.findByRole("heading", { name: "Kit 2" });
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: { action: "td27_read", target: emulatorTarget, kit: 2 },
    });
  });

  it("clears sandbox state and its plan before switching to native MIDI", async () => {
    mockedInvoke
      .mockResolvedValueOnce(reply(snapshot))
      .mockResolvedValueOnce(reply(plan));
    await readEmulator();
    await planTempo();
    expect(screen.getByRole("button", { name: /Apply in sandbox/ })).toBeEnabled();
    fireEvent.click(screen.getByRole("radio", { name: "Native MIDI" }));
    expect(screen.queryByText("1 planned write")).not.toBeInTheDocument();
    expect(screen.queryByRole("textbox", { name: "Kit name" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Apply to TD-27/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Read kit" })).toBeDisabled();
    expect(mockedInvoke).toHaveBeenCalledTimes(2);
  });

  it("reports a core read error without creating editable device state", async () => {
    mockedInvoke.mockResolvedValueOnce({
      api_version: 1,
      ok: false,
      error: { code: "midi_timeout", message: "No matching identity reply" },
    });
    render(<Td27Panel />);
    fireEvent.click(screen.getByRole("radio", { name: "Sandbox (TD-27 emulator)" }));
    fireEvent.click(screen.getByRole("button", { name: "Read kit" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "No matching identity reply (midi_timeout)",
    );
    expect(
      screen.queryByRole("textbox", { name: "Kit name" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Readback verified")).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Apply in sandbox/ }),
    ).toBeDisabled();
  });

  it("preserves the draft after an incomplete apply and requires a fresh read when no snapshot is returned", async () => {
    const partial: Td27Report = {
      status: "partial",
      written: 1,
      verified: 0,
      total: 1,
      message: "Disconnected before readback",
      snapshot: null,
    };
    mockedInvoke
      .mockResolvedValueOnce(reply(snapshot))
      .mockResolvedValueOnce(reply(plan))
      .mockResolvedValueOnce(reply(partial));
    await readEmulator();
    await planTempo();
    fireEvent.click(screen.getByRole("button", { name: /Apply in sandbox/ }));
    await screen.findByText("Apply partial");
    expect(screen.getByText("1 written · 0 verified · 1 total")).toBeVisible();
    expect(screen.getByRole("spinbutton", { name: "Tempo" })).toHaveValue(125);
    expect(screen.getByRole("spinbutton", { name: "Tempo" })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Plan changes/ })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: /Apply in sandbox/ }),
    ).toBeDisabled();
    expect(screen.queryByText("Readback verified")).not.toBeInTheDocument();
    expect(
      screen.getByText(
        "Read the kit again to establish its current state. Your draft remains local.",
      ),
    ).toBeVisible();
  });

  it("requires a new plan when the packet scenario changes", async () => {
    mockedInvoke
      .mockResolvedValueOnce(reply(snapshot))
      .mockResolvedValueOnce(reply(plan))
      .mockResolvedValueOnce(reply(plan));
    await readEmulator();
    await planTempo();
    fireEvent.change(
      screen.getByRole("combobox", { name: "Sandbox scenario" }),
      { target: { value: "readback_mismatch" } },
    );
    expect(
      screen.getByRole("button", { name: /Apply in sandbox/ }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: /Plan changes/ }));
    await screen.findByText("1 planned write");
    expect(mockedInvoke).toHaveBeenLastCalledWith("dispatch", {
      request: {
        action: "td27_plan",
        baseline: snapshot,
        changes: { "kit.tempo": 125 },
      },
    });
  });
});
