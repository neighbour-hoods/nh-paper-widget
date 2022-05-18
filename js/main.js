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
        sm_label_map: `\
{
  "0": "unreviewed",
  "1": "accepted"
}`,
      },
      sm_init_s: {
      },
      sm_comp_s: {
      },
      sm_label_map_s: {
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

    this.get_sm_init_and_comp_and_label_map();

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
    async get_sm_init_and_comp_and_label_map() {
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
        this.sm_label_map_s[label] = await this.hcInfo.appWs.callZome({
          cap: null,
          cell_id: cell_id,
          zome_name: 'paperz_main_zome',
          fn_name: 'get_sm_label_map',
          payload: label,
          provenance: cell_id[1],
        });
      }
      console.log("sm_init_s:");
      console.log(this.sm_init_s);
      console.log("sm_comp_s:");
      console.log(this.sm_comp_s);
      console.log("sm_label_map_s:");
      console.log(this.sm_label_map_s);
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

      await asyncForEach(this.paperz, async (ele, index) => {
        let annotationz = await this.hcInfo.appWs.callZome({
          cap: null,
          cell_id: cell_id,
          zome_name: 'paperz_main_zome',
          fn_name: 'get_annotations_for_paper',
          payload: ele[0],
          provenance: cell_id[1],
        });
        await asyncForEach(annotationz, async (ele, index) => {
          let ann_eh = ele[0];
          let sm_data = await this.hcInfo.appWs.callZome({
            cap: null,
            cell_id: cell_id,
            zome_name: 'paperz_main_zome',
            fn_name: 'get_sm_data_for_eh',
            payload: [ann_eh, null],
            provenance: cell_id[1],
          });
          console.log("sm_data");
          console.log(sm_data);
          annotationz[index].push(sm_data);
        });
        console.log("annotationz");
        console.log(annotationz);
        this.paperz[index].annotationz = annotationz;
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
      this.get_sm_init_and_comp_and_label_map();
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
      this.get_sm_init_and_comp_and_label_map();
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
    },
    async handleCreateAnnotationSubmit(paper_ref, evt) {
      let obj = {
        paper_ref: paper_ref,
        page_num: evt.target.elements.page_num.valueAsNumber,
        paragraph_num: evt.target.elements.paragraph_num.valueAsNumber,
        what_it_says: evt.target.elements.what_it_says.value,
        what_it_should_say: evt.target.elements.what_it_should_say.value,
      };

      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;
      let [eh, hh] = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'paperz_main_zome',
        fn_name: 'create_annotation',
        payload: obj,
        provenance: cell_id[1],
      });
      console.log("handleCreateAnnotationSubmit:");
      console.log(eh);
      console.log(hh);

      this.get_paperz();
    },
    async handleStepSm(ann_eh, evt) {
      console.log("handleStepSm:");
      console.log(ann_eh);
      console.log(evt);

      let obj = {
        target_eh: ann_eh,
        label: "annotationz",
        act: evt.target.elements.action.value,
      };
      console.log(obj);

      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;
      await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'paperz_main_zome',
        fn_name: 'step_sm',
        payload: obj,
        provenance: cell_id[1],
      });
      this.get_paperz();
    },
    async set_sm_label_map() {
      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;
      let obj = JSON.parse(this.sm_submit.sm_label_map);
      console.log("obj:");
      console.log(obj);
      let res = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'paperz_main_zome',
        fn_name: 'set_sm_label_map',
        payload: ["annotationz", obj],
        provenance: cell_id[1],
      });
      console.log(res);
      this.get_sm_init_and_comp_and_label_map();
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

async function asyncForEach(array, callback) {
  for (let index = 0; index < array.length; index++) {
    await callback(array[index], index, array);
  }
}
