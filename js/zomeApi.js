import { getCellId } from "./hcClient";

export default class ZomeApi {
  client

  constructor(holochainCellClient) {
    this.client = holochainCellClient
  }

  async set_hub_cell_id(payload) {
    return await this.client.callZome('paperz_main_zome',  'set_hub_cell_id', payload);
  }

  async get_hub_cell_id() {
    return await this.client.callZome('paperz_main_zome',  'get_hub_cell_id');
  }

  /// Plain holochain widget calls
  async get_all_paperz() {
    return await this.client.callZome('paperz_main_zome',  'get_all_paperz');
  }

  async get_annotations_for_paper(ele) {
    return await this.client.callZome('paperz_main_zome',  'get_annotations_for_paper', ele);
  }

  async upload_paper(payload) {
    return await this.client.callZome('paperz_main_zome',  'upload_paper', payload);
  }

  // Holochain call with sensemaker bridge call
  async create_annotation(payload) {
    return await this.client.callZome('paperz_main_zome',  'create_annotation', payload);
  }

  // Sensemaker bridge calls
  async get_sm_init(payload) {
    return await this.client.callZome('paperz_main_zome', 'get_state_machine_init', payload);
  };

  async get_sm_comp(payload) {
    return await this.client.callZome('paperz_main_zome',  'get_state_machine_comp', payload);
  }

  async get_sm_data_for_eh(payload) {
    return await this.client.callZome('paperz_main_zome',  'get_state_machine_data', payload);
  }

  async set_sm_comp_se_eh(payload) {
    return await this.client.callZome('paperz_main_zome',  'set_state_machine_comp', payload);
  }

  async set_sm_init_se_eh(payload) {
    return await this.client.callZome('paperz_main_zome',  'set_state_machine_init', payload);
  }

  async step_sm(payload) {
    return await this.client.callZome('paperz_main_zome',  'step_sm', payload);
  }
}

