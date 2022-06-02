import { HolochainClient } from "@holochain-open-dev/cell-client";

let cellClient;
let agentPubKey;
let cellId;

export async function getHubCellData() {
  const installed_app_id = "test-app";
  const hcClient = await HolochainClient.connect(
    "ws://localhost:8888",
    installed_app_id
  );
  let client = hcClient;
  const roleId = "main";
  // Find the cell you want to make the call to
  const cellData = client.cellDataByRoleId(roleId);

  return cellData;
}
