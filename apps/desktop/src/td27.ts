import { dispatch } from "./core";

export interface Td27Snapshot {
  format_version: 1;
  simulated: boolean;
  endpoint?: { input: string; output: string } | null;
  identity: {
    device_id: number;
    software_revision: number[];
    firmware: string;
  };
  kit: number;
  blocks: { address: number[]; data: number[] }[];
  parameters: Record<string, unknown>;
}

export type Td27Target =
  | {
      transport: "native";
      input: string;
      output: string;
      device_id: number;
      firmware: string;
    }
  | {
      transport: "emulator";
      state?: Td27Snapshot;
      device_id?: number;
      firmware?: string;
      fault?: string;
    };

export interface MidiPorts {
  inputs: string[];
  outputs: string[];
}
export type Td27Changes = Record<string, string | number | boolean>;
export interface Td27Plan {
  format_version: number;
  baseline: Td27Snapshot;
  baseline_hash: string;
  changes: Td27Changes;
  writes: {
    key: string;
    address: number[];
    before: number[];
    after: number[];
  }[];
}
export interface Td27Report {
  status: string;
  written: number;
  verified: number;
  total: number;
  message: string;
  snapshot: Td27Snapshot | null;
}

export const td27Core = {
  ports: () => dispatch<MidiPorts>({ action: "midi_ports" }),
  read: (target: Td27Target, kit: number) =>
    dispatch<Td27Snapshot>({ action: "td27_read", target, kit }),
  plan: (baseline: Td27Snapshot, changes: Td27Changes) =>
    dispatch<Td27Plan>({ action: "td27_plan", baseline, changes }),
  apply: (target: Td27Target, plan: Td27Plan) =>
    dispatch<Td27Report>({ action: "td27_apply", target, plan }),
};

export type Td27Client = typeof td27Core;
