// 🤷‍️, from \/
// https://github.com/fengyuanchen/vue-feather/issues/8
import { createApp } from 'vue/dist/vue.esm-bundler';
import { AdminWebsocket, AppWebsocket, InstalledAppInfo } from '@holochain/conductor-api';

const STATUS_INITIAL = 0, STATUS_SAVING = 1, STATUS_SUCCESS = 2, STATUS_FAILED = 3;

const App = {
  name: 'app',
  data() {
    return {
      uploadError: null,
      currentStatus: null,
      hcInfo: null,
      memez: [],
    }
  },
  async created () {
    let appWs = await AppWebsocket.connect('ws://localhost:9999');
    let adminWs = await AdminWebsocket.connect('ws://localhost:9000');
    let agentPk = await adminWs.generateAgentPubKey();
    let hcInfo = {
        adminWs: adminWs,
        appWs: appWs,
        agentPk: agentPk,
    };
    this.hcInfo = hcInfo;

    this.get_memez()
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
    async save(file) {
      // upload data to the server
      this.currentStatus = STATUS_SAVING;

      let data = await getBase64(file);
      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;
      const res = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'memez_main_zome',
        fn_name: 'upload_meme',
        payload: {
          params_string: data
        },
        provenance: cell_id[1],
      });
      this.currentStatus = STATUS_INITIAL;
      // TODO handle `res` error cases

      this.get_memez()
    },
    async get_memez() {
      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;
      const res = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'memez_main_zome',
        fn_name: 'get_all_meme_strings',
        payload: {
          params_string: ""
        },
        provenance: cell_id[1],
      });
      console.log(res);
      this.memez = res;
    },
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
