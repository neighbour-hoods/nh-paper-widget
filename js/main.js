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
      sm_submit: {
        sm_init: {
          label: "annotationz",
          expr_str: "0",
        },
        sm_comp: {
          label: "annotationz",
          expr_str: `\
(lam [st act]
  (if (== st 0)
    (if (== act 0)
      0
      (if (== act 1)
        1
        st))
    (if (== st 1)
      (if (== act 0)
        0
        (if (== act 1)
          1
          st))
      st)))`,
        },
      },
      sm_init_s: {
      },
      sm_comp_s: {
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

    this.get_sm_init_and_comp_s();

    this.get_paperz();
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
    async handleHcPortSubmit() {
      localStorage.setItem('hcAppPort', this.hcAppPort);
      localStorage.setItem('hcAdminPort', this.hcAdminPort);
      window.location.reload()
    },
    async get_sm_init_and_comp_s() {
      const labels = ["annotationz"];

      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;

      for (var i = 0; i < labels.length; i++) {
        let label = labels[i];

        this.sm_init_s[label] = await this.hcInfo.appWs.callZome({
          cap: null,
          cell_id: cell_id,
          zome_name: 'paperz_main_zome',
          fn_name: 'get_sm_init',
          payload: label,
          provenance: cell_id[1],
        });
        this.sm_comp_s[label] = await this.hcInfo.appWs.callZome({
          cap: null,
          cell_id: cell_id,
          zome_name: 'paperz_main_zome',
          fn_name: 'get_sm_comp',
          payload: label,
          provenance: cell_id[1],
        });
      }
      console.log("sm_init_s:");
      console.log(this.sm_init_s);
      console.log("sm_comp_s:");
      console.log(this.sm_comp_s);
    },
    async get_paperz() {
      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;
      this.paperz = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'paperz_main_zome',
        fn_name: 'get_all_papers',
        payload: null,
        provenance: cell_id[1],
      });
      console.log("paperz:");
      console.log(this.paperz);
    },
    async set_sm_init() {
      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;
      let res = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'paperz_main_zome',
        fn_name: 'set_sm_init_se_eh',
        payload: [this.sm_submit.sm_init.label, this.sm_submit.sm_init.expr_str],
        provenance: cell_id[1],
      });
      console.log(res);
      this.get_sm_init_and_comp_s();
    },
    async set_sm_comp() {
      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;
      let res = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'paperz_main_zome',
        fn_name: 'set_sm_comp_se_eh',
        payload: [this.sm_submit.sm_comp.label, this.sm_submit.sm_comp.expr_str],
        provenance: cell_id[1],
      });
      console.log(res);
      this.get_sm_init_and_comp_s();
    },
    async handlePaperSubmit(evt) {
      this.currentStatus = STATUS_SAVING;
      console.log("handlePaperSubmit");
      console.log(evt);
      let file = evt.target.files[0];
      let obj = {
        filename: file.name,
        blob_str: await getBase64(file),
      };
      console.log(obj);

      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;
      let hh = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'paperz_main_zome',
        fn_name: 'upload_paper',
        payload: obj,
        provenance: cell_id[1],
      });
      console.log(hh);
      this.currentStatus = STATUS_INITIAL;

      this.get_paperz();
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
