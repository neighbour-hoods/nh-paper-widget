import { HolochainClient } from "@holochain-open-dev/cell-client";

let cellClient;
let agentPubKey;
let cellId;

export async function setupClient() {
  const installed_app_id = "test-app"; // this is default -> https://github.com/holochain/holochain/blob/2d9401a5e0f74934195a0bf02ca198679b53089d/crates/hc_sandbox/src/cli.rs
  const hcClient = await HolochainClient.connect(
    "ws://localhost:9999",
    installed_app_id
  );
  let client = hcClient;
  const roleId = "main";
  // Find the cell you want to make the call to
  const cellData = client.cellDataByRoleId(roleId);

  cellClient = client.forCell(cellData);
  agentPubKey = client.agentPubKey;

  return cellClient;
}

export function getCellClient() {
  return cellClient;
}

export function getAgentPubKey() {
  return agentPubKey;
}

// export function getCellId() {
//   console.log(cellId);
//   return cellId;
// }

