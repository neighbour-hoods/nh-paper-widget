import { AdminWebsocket, AppWebsocket, InstalledAppInfo } from '@holochain/client';

export async function setupClient(appPort, adminPort) {
  let appWs = await AppWebsocket.connect('ws://localhost:' + appPort.toString());
  let adminWs = await AdminWebsocket.connect('ws://localhost:' + adminPort.toString());
  let agentPk = await adminWs.generateAgentPubKey();
  let hcInfo = {
      adminWs,
      appWs,
      agentPk,
  };

  return hcInfo;
}
