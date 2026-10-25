import { invoke } from "@tauri-apps/api/core";

export interface Pad {
  instrument: string;
  tuning: number;
  muffling: number;
  volume: number;
  pan: number;
  sub_instrument: string | null;
  layer_level: number;
}

export interface Kit {
  name: string;
  pads: Record<string, Pad>;
  effects: Record<string, unknown>;
  opaque: Record<string, number[]>;
}

export interface Document {
  format_version: 1;
  profile_id: string;
  document_id: string;
  kits: Record<string, Kit>;
  slots: string[];
  setlists: { name: string; slots: number[] }[];
}

export interface DeviceIdentity {
  device_id: string;
  profile_id: string;
  simulated: boolean;
}

export interface Snapshot {
  identity: DeviceIdentity;
  document: Document;
}
export interface Change {
  path: string;
  before: unknown | null;
  after: unknown | null;
}
export interface ChangePlan {
  format_version: 1;
  expected_device: DeviceIdentity;
  baseline_hash: string;
  desired_hash: string;
  baseline: Document;
  desired: Document;
  changes: Change[];
}

export interface ApplyReport {
  status: "verified" | "conflict" | "partial" | "failed";
  written: number;
  verified: number;
  total: number;
  message: string;
  snapshot: Snapshot;
  changes: Change[];
}

export interface Demo {
  document: Document;
  snapshot: Snapshot;
  profile: {
    id: string;
    name: string;
    simulated: true;
    hardware_writes: false;
    instruments: string[];
  };
  hardware_status: string;
}

export type Operation =
  | { kind: "rename_kit"; kit_id: string; name: string }
  | {
      kind: "set_parameter";
      kit_id: string;
      pad: string;
      parameter: string;
      value: number | string | null;
    }
  | {
      kind: "copy_pad";
      source_kit_id: string;
      source_pad: string;
      target_kit_id: string;
      target_pad: string;
    }
  | {
      kind: "copy_kit";
      source_kit_id: string;
      target_slot: number;
      new_kit_id: string;
    }
  | { kind: "reorder_kits"; order: string[] };

export interface CoreClient {
  demo(): Promise<Demo>;
  edit(document: Document, operation: Operation): Promise<Document>;
  plan(baseline: Snapshot, desired: Document): Promise<ChangePlan>;
  apply(
    device: Snapshot,
    plan: ChangePlan,
    fault: "none" | "disconnect" | "readback_mismatch",
  ): Promise<ApplyReport>;
  load(path: string): Promise<Document>;
  save(
    path: string,
    document: Document,
  ): Promise<{ path: string; files: number }>;
}

interface Response<T> {
  api_version: 1;
  ok: boolean;
  result?: T;
  error?: { code: string; message: string };
}

export async function dispatch<T>(
  request: Record<string, unknown>,
): Promise<T> {
  const response = await invoke<Response<T>>("dispatch", { request });
  if (response.api_version !== 1)
    throw new Error("The core API version is not supported.");
  if (!response.ok || response.result === undefined) {
    throw new Error(
      response.error
        ? `${response.error.message} (${response.error.code})`
        : "The core returned no result.",
    );
  }
  return response.result;
}

export const core: CoreClient = {
  demo: () => dispatch({ action: "demo" }),
  edit: (document, operation) =>
    dispatch({ action: "edit", document, operation }),
  plan: (baseline, desired) => dispatch({ action: "plan", baseline, desired }),
  apply: (device, plan, fault) =>
    dispatch({ action: "simulate_apply", device, plan, fault }),
  load: (path) => dispatch({ action: "load", path }),
  save: (path, document) => dispatch({ action: "save", path, document }),
};
