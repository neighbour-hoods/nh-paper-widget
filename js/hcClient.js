import { AdminWebsocket, AppWebsocket, InstalledAppInfo } from '@holochain/client';

export class HcClient {
  constructor(adminWs, appWs, agentPk) {
    this.adminWs = adminWs;
    this.appWs = appWs;
    this.agentPk = agentPk;
  }

  static async initialize(appPort, adminPort) {
      let appWs = await AppWebsocket.connect('ws://localhost:' + appPort.toString());
      let adminWs = await AdminWebsocket.connect('ws://localhost:' + adminPort.toString());
      let agentPk = await adminWs.generateAgentPubKey();
      return new HcClient(adminWs, appWs, agentPk);
  }

  async callZome(fn_name, payload) {
    let info = await this.appWs.appInfo({
      installed_app_id: 'test-app',
    });
    const cell_id = info.cell_data[0].cell_id;
    await this.appWs.callZome({
      cap: null,
      cell_id: cell_id,
      zome_name: 'paperz_main_zome',
      fn_name,
      payload,
      provenance: cell_id[1],
    })
  }

  async set_hub_cell_id(payload) {
    console.log('set_hub_cell_id: payload: ', payload);
    return await this.callZome('set_hub_cell_id', payload);
  }

  async get_hub_cell_id() {
    return await this.callZome('get_hub_cell_id', null);
  }

  /// Plain holochain widget calls
  async get_all_paperz() {
    return await this.callZome('get_all_paperz', null);
  }

  async get_annotations_for_paper(ele) {
    return await this.callZome('get_annotations_for_paper', ele);
  }

  async upload_paper(payload) {
    return await this.callZome('upload_paper', payload);
  }

  // Holochain call with sensemaker bridge call
  async create_annotation(payload) {
    return await this.callZome('create_annotation', payload);
  }

  // Sensemaker bridge calls
  async get_sm_init(payload) {
    return await this.callZome('get_state_machine_init', payload);
  };

  async get_sm_comp(payload) {
    return await this.callZome('get_state_machine_comp', payload);
  }

  async get_sm_data_for_eh(payload) {
    return await this.callZome('get_state_machine_data', payload);
  }

  async set_sm_comp_se_eh(payload) {
    return await this.callZome('set_state_machine_comp', payload);
  }

  async set_sm_init_se_eh(payload) {
    return await this.callZome('set_state_machine_init', payload);
  }

  async step_sm(payload) {
    return await this.callZome('step_sm', payload);
  }
}
