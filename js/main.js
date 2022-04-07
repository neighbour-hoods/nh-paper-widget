// 🤷‍️, from \/
// https://github.com/fengyuanchen/vue-feather/issues/8
import { createApp } from 'vue/dist/vue.esm-bundler';
import { AdminWebsocket, AppWebsocket, InstalledAppInfo } from '@holochain/conductor-api';

const STATUS_INITIAL = 0, STATUS_SAVING = 1, STATUS_SUCCESS = 2, STATUS_FAILED = 3;

const App = {
  name: 'paperz',
  data() {
    let hcAppPort = localStorage.getItem('hcAppPort');
    if (hcAppPort === null) {
      hcAppPort = 9999;
      localStorage.setItem('hcAppPort', hcAppPort);
    }
    let hcAdminPort = localStorage.getItem('hcAdminPort');
    if (hcAdminPort === null) {
      hcAdminPort = 9000;
      localStorage.setItem('hcAdminPort', hcAdminPort);
    }
    return {
      hcAppPort,
      hcAdminPort,
      uploadError: null,
      currentStatus: null,
      hcInfo: null,
      paperz: [],
      annotationz: [],
      sm_init: {
      },
      sm_comp: {
      },
    }
  },
  async created () {
    let appWs = await AppWebsocket.connect('ws://localhost:' + this.hcAppPort.toString());
    let adminWs = await AdminWebsocket.connect('ws://localhost:' + this.hcAdminPort.toString());
    let agentPk = await adminWs.generateAgentPubKey();
    let hcInfo = {
        adminWs: adminWs,
        appWs: appWs,
        agentPk: agentPk,
    };
    this.hcInfo = hcInfo;
    console.log("hcInfo:");
    console.log(hcInfo);

    let info = await this.hcInfo.appWs.appInfo({
      // TODO figure out why this works... it shouldn't, I think?
      installed_app_id: 'test-app',
    });
    const cell_id = info.cell_data[0].cell_id;

    const labels = ["annotationz"];

    for (var i = 0; i < labels.length; i++) {
      let label = labels[i];

      this.sm_init[label] = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'paperz_main_zome',
        fn_name: 'get_sm_init_se_eh',
        payload: label,
        provenance: cell_id[1],
      });
      this.sm_comp[label] = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'paperz_main_zome',
        fn_name: 'get_sm_comp_se_eh',
        payload: label,
        provenance: cell_id[1],
      });
    }
    console.log("sm_init:");
    console.log(this.sm_init);
    console.log("sm_comp:");
    console.log(this.sm_comp);
  },
  computed: {
    isInitial() {
      return this.currentStatus === STATUS_INITIAL;
    },
    isSaving() {
      return this.currentStatus === STATUS_SAVING;
    },
    isSuccess() {
      return this.currentStatus === STATUS_SUCCESS;
    },
    isFailed() {
      return this.currentStatus === STATUS_FAILED;
    }
  },
  methods: {
    reset() {
      // reset form to initial state
      this.currentStatus = STATUS_INITIAL;
      this.uploadError = null;
    },
    async save(paper) {
      console.log("paper:");
      console.log(paper);
    },
    async set_sm_init(init_val) {
      console.log("set_sm_init: ");
      console.log(init_val);
    },
    async set_sm_comp(comp_val) {
      console.log("set_sm_comp: ");
      console.log(comp_val);
    }
  },
  mounted() {
    this.reset();
  },
}

const app = createApp(App)
app.mount('#app')


////////////////////////////////////////////////////////////////////////////////
// helpers
////////////////////////////////////////////////////////////////////////////////

function getBase64(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = () => resolve(reader.result);
    reader.onerror = error => reject(error);
  });
}
