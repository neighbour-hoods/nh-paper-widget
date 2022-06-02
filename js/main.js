// 🤷‍️, from \/
// https://github.com/fengyuanchen/vue-feather/issues/8
import { createApp } from 'vue/dist/vue.esm-bundler';
import { setupClient } from './hcClient';
import { getHubCellData } from './hubClient';
import ZomeApi from './zomeApi';

const STATUS_INITIAL = 0, STATUS_SAVING = 1, STATUS_SUCCESS = 2, STATUS_FAILED = 3;

const App = {
  name: 'paperz',
  data() {
    return {
      zomeApi: null,
      uploadError: null,
      currentStatus: null,
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
    // async handleHcPortSubmit() {
    //   localStorage.setItem('hcAppPort', this.hcAppPort);
    //   localStorage.setItem('hcAdminPort', this.hcAdminPort);
    //   window.location.reload()
    // },
    async get_sm_init_and_comp_s() {
      const labels = ["annotationz"];

      for (var i = 0; i < labels.length; i++) {
        let label = labels[i];
        let res = await this.zomeApi.get_sm_init(label);
        console.log('res', res);
        this.sm_init_s[label] = await this.zomeApi.get_sm_init(label);
        console.log("sm_init_s", this.sm_init_s[label]);
        this.sm_comp_s[label] = await this.zomeApi.get_sm_comp(label);
        console.log("sm_comp_s", this.sm_comp_s[label]);
      }

      console.log("sm_init_s: ", this.sm_init_s);
      console.log("sm_comp_s:", this.sm_comp_s);
    },
    async get_paperz() {
      console.log("##### GETTING PAPERZ #####");
      this.paperz = await this.zomeApi.get_all_paperz();
      console.log("got all paperz: ", this.paperz);
      // I think we can turn this into a tree structure using Path on the backend
      // Will be a bit of legwork to get going but would remove the need for looped callback
      // patterns like below.
      // How often will context-resource-sensemaker data be representable by a tree?
      console.log("Starting 1st async, for each paper, get annotations");
      await asyncForEach(this.paperz, async (ele, index) => {
        // for each paper, get annotations for paper
        let annotationz = await this.zomeApi.get_annotations_for_paper(ele);
        console.log("Annotationz for paper: ", annotationz);

        // for each annotation get all sensemaker data
        console.log("Starting 2nd async forEach, get sensemaker");
        await asyncForEach(annotationz, async (ele, index) => {
          console.log('getting sm_data');
          let sm_data = await this.zomeApi.get_sm_data_for_eh([ele[0], null]);
          console.log("sm_data: ", sm_data);
          annotationz[index].push(sm_data);
        });
        console.log("annotationz: ", annotationz);
        this.paperz[index].annotationz = annotationz;
      });
      console.log("paperz: ", this.paperz);
      console.log("##### DONE GETTING PAPERZ #####");
    },
    // initialize sense maker state machine to
    async set_sm_init() {
      let payload = [this.sm_submit.sm_init.label, this.sm_submit.sm_init.expr_str];
      let res = await this.zomeApi.set_sm_init_se_eh(payload);
      console.log("set_sm_init res: ", res);
      this.get_sm_init_and_comp_s();
    },
    async set_sm_comp() {
      let payload = [this.sm_submit.sm_comp.label, this.sm_submit.sm_comp.expr_str];
      let res = await this.zomeApi.set_sm_comp_se_eh(payload);

      console.log("set_sm_comp res: ", res);
      this.get_sm_init_and_comp_s();
    },
    async handlePaperSubmit(evt) {
      this.currentStatus = STATUS_SAVING;
      console.log("handlePaperSubmit: ", evt);
      let file = evt.target.files[0];
      let obj = {
        filename: file.name,
        blob_str: await getBase64(file),
      };
      console.log(obj);

      let hh = await this.zomeApi.upload_paper(obj);
      console.log('Paper HeaderHash: ', hh);
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

      let [eh, hh] = await this.zomeApi.create_annotation(obj);
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

      await this.zomeApi.step_sm(obj);
    }
  },

////////////////////////////////////////////////////////////////////////////////
// lifecycle hooks
////////////////////////////////////////////////////////////////////////////////
  async beforeMount () {
    console.log('beforeMount');
    console.log('BeforeCreate');
    let client = await setupClient();
    this.zomeApi = new ZomeApi(client);
    console.log('zomeApi: ', this.zomeApi);

    let hubCellData = await getHubCellData();
    console.log('hubClient cellData:',  hubCellData);
    await this.zomeApi.set_hub_cell_id(hubCellData.cell_id);
    let retrieved_cell_id = await this.zomeApi.get_hub_cell_id();
    console.log('retrieved_cell_id: ', retrieved_cell_id);

    this.get_sm_init_and_comp_s();
    this.get_paperz();
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
