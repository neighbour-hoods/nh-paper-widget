import { HolochainClient } from "@holochain-open-dev/cell-client";

let client;
let agentPubKey;
let cellId;

export async function setupClient() {
  const installed_app_id = "test-app"; // this is default -> https://github.com/holochain/holochain/blob/2d9401a5e0f74934195a0bf02ca198679b53089d/crates/hc_sandbox/src/cli.rs
  const client = await HolochainClient.connect(
    "ws://localhost:9999",
    installed_app_id
  );
  client = client;
  agentPubKey = client.agentPubKey;
  cellId = client.cellId;

  return client;
}

export function getClient() {
  return this.client;
}

export function getAgentPubKey() {
  return this.agentPubKey;
}

export function getCellId() {
  return this.cellId;
}

