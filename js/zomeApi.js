import { getCellId } from "./hcClient";

export default class ZomeApi {
  client

  constructor(holochainClient) {
    this.client = holochainClient
  }

  async get_sm_init(label) {
    return await this.client.callZome(getCellId(), 'paperz_main_zome', 'get_sm_init', label);
  };

  async get_sm_comp(label) {
    return await this.client.callZome(getCellId(), 'paperz_main_zome',  'get_sm_comp', label);
  }

  async get_all_paperz() {
    return await this.client.callZome(getCellId(), 'paperz_main_zome',  'get_all_papers');
  }

  async get_annotations_for_paper(ele) {
    return await this.client.callZome(getCellId(), 'paperz_main_zome',  'get_annotations_for_paper', ele);
  }

  async get_sm_data_for_eh(payload) {
    return await this.client.callZome(getCellId(), 'paperz_main_zome',  'get_sm_data_for_eh', payload);
  }

  async upload_paper(payload) {
    return await this.client.callZome(getCellId(), 'paperz_main_zome',  'upload_paper', payload);
  }

  async create_annotation(payload) {
    return await this.client.callZome(getCellId(), 'paperz_main_zome',  'create_annotation', payload);
  }

  async step_sm(payload) {
    return await this.client.callZome(getCellId(), 'paperz_main_zome',  'step_sm', payload);
  }

  async set_sm_comp_se_eh(payload) {
    return await this.client.callZome(getCellId(), 'paperz_main_zome',  'set_sm_comp_se_eh', payload);
  }

  async set_sm_init_se_eh(payload) {
    return await this.client.callZome(getCellId(), 'paperz_main_zome',  'set_sm_init_se_eh', payload);
  }
}

