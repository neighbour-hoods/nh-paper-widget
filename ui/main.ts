// 🤷‍️, from \/
// https://github.com/fengyuanchen/vue-feather/issues/8
import { createApp } from 'vue';
import { ZomeApi } from './zomeApi';
import {getLogger} from "./LogConfig";
import { Annotation, Paper } from './types/paperz';
import { StepStateMachineInput } from './types/sensemaker';
import { EntryHashB64 } from "@holochain-open-dev/core-types"

const STATUS_INITIAL = 0, STATUS_SAVING = 1, STATUS_SUCCESS = 2, STATUS_FAILED = 3;

const ANNOTATION_PATH: string = "widget.paperz.annotationz";

const App = {
  name: 'paperz',
  async data() {
    let hcAppPort: string | null = localStorage.getItem('hcAppPort');
    if (hcAppPort === null) {
      hcAppPort = '9999';
      localStorage.setItem('hcAppPort', hcAppPort);
    }
    let hcAdminPort: string | null = localStorage.getItem('hcAdminPort');
    if (hcAdminPort === null) {
      hcAdminPort = '9000';
      localStorage.setItem('hcAdminPort', hcAdminPort);
    }

    let zomeApi: ZomeApi =  await ZomeApi.initialize(hcAppPort, hcAdminPort);

    const logger = getLogger("main");
    return {
      hcAppPort,
      hcAdminPort,
      logger,
      uploadError: null,
      currentStatus: null,
      zomeApi,
      paperz: [],
      annotationz: [],
      sm_submit: {
        path_string: ANNOTATION_PATH,
        sm_init: {
          expr_str: "0",
        },
        sm_comp: {
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
    isInitial(): boolean {
      return this.currentStatus === STATUS_INITIAL;
    },
    isSaving(): boolean {
      return this.currentStatus === STATUS_SAVING;
    },
    isSuccess(): boolean {
      return this.currentStatus === STATUS_SUCCESS;
    },
    isFailed(): boolean {
      return this.currentStatus === STATUS_FAILED;
    }
  },
  methods: {
    reset(): void {
      // reset form to initial state
      this.currentStatus = STATUS_INITIAL;
      this.uploadError = null;
    },
    async handleHcPortSubmit(): Promise<void> {
      localStorage.setItem('hcAppPort', this.hcAppPort);
      localStorage.setItem('hcAdminPort', this.hcAdminPort);
      window.location.reload()
    },
    async get_sm_init_and_comp_s(): Promise<void> {
      this.logger.debug('get_sm_init_and_comp_s...');
      const paths: Array<string> = [ANNOTATION_PATH];

      for (var i = 0; i < paths.length; i++) {
        let path = paths[i];
        this.sm_init_s[path] = await this.zomeApi.get_state_machine_init(path);
        this.sm_comp_s[path] = await this.zomeApi.get_state_machine_comp(path);
      }
    },

    async get_paperz(): Promise<void> {
      this.paperz = await this.zomeApi.get_all_paperz();
      
      await asyncForEach(this.paperz, async (paper: Array<Paper>, index: number) => {
        // get annotations for each paper
        let annotationz = await this.zomeApi.get_annotations_for_paper(paper[0]);

        // get sensemaker entries for annotation
        await asyncForEach(annotationz, async (annotation: Array<Annotation> , index: number) => {
          let sm_data = await this.zomeApi.get_state_machine_data(annotation[0]);
          annotationz[index].push(sm_data);
        });
        this.paperz[index].annotationz = annotationz;
      });
    },
    // initialize sense maker state machine to
    async set_sm_init(): Promise<void> {
      let payload = [this.sm_submit.path_string, this.sm_submit.sm_init.expr_str];
      await this.zomeApi.set_state_machine_init(payload);
      this.get_sm_init_and_comp_s();
    },
    async set_sm_comp() {
      let payload = [this.sm_submit.path_string, this.sm_submit.sm_comp.expr_str];
      await this.zomeApi.set_state_machine_comp(payload);
      this.get_sm_init_and_comp_s();
    },
    // TODO: get a type for event.
    async handlePaperSubmit(event: any): Promise<void> {
      this.currentStatus = STATUS_SAVING;
      let file = event.target.files[0];
      let obj = {
        filename: file.name,
        blob_str: await getBase64(file),
      };
      await this.zomeApi.upload_paper(obj);
      this.currentStatus = STATUS_INITIAL;
      await this.get_paperz();
    },
    async handleCreateAnnotationSubmit(paper_ref: Blob, event: any): Promise<void> {
      let obj = {
        paper_ref: paper_ref,
        page_num: event.target.elements.page_num.valueAsNumber,
        paragraph_num: event.target.elements.paragraph_num.valueAsNumber,
        what_it_says: event.target.elements.what_it_says.value,
        what_it_should_say: event.target.elements.what_it_should_say.value,
      };

      let [eh, hh] = await this.zomeApi.create_annotations(obj);
      await this.get_paperz();
    },
    async handleStepSm(ann_eh: EntryHashB64, event: any | null): Promise<void> {
      await this.zomeApi.step_sm({
        path: ANNOTATION_PATH,
        entry_hash: ann_eh,
        action: event.target.elements.action.value
      } as StepStateMachineInput);

      await this.get_paperz();
    }
  },

////////////////////////////////////////////////////////////////////////////////
// lifecycle hooks
////////////////////////////////////////////////////////////////////////////////
  async beforeMount () {
    this.zomeApi = await ZomeApi.initialize(this.hcAppPort, this.hcAdminPort);

    let admin = this.zomeApi.adminWs;
    let cells = await admin.listCellIds();

    const installed_app_id = 'sensemaker';
    if (cells.length == 1) {
      const sensemakerDnaHash = await admin.registerDna({
        path: './result/social_sensemaker.dna',
      });
      const installedApp = await admin.installApp({
        installed_app_id,
        agent_key: this.zomeApi.cellId[1],
        dnas: [{ hash: sensemakerDnaHash, role_id: 'thedna' }],
      });
      console.log('installedApp: ', installedApp);
      const startApp1 = await admin.enableApp({ installed_app_id });
      console.log('startApp1: ', startApp1);

      const sensemakerCell = installedApp.cell_data[0].cell_id;
      console.log('setting sensemakerCell: ', sensemakerCell);
      let res = await this.zomeApi.set_sensemaker_cell_id(sensemakerCell);
      console.log('set_sensemaker_cell_id: ', res);
    }

    await this.get_sm_init_and_comp_s();
    await this.get_paperz();
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

function getBase64(file: Blob): Promise<string | ArrayBuffer> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = () => resolve(reader.result!);
    reader.onerror = error => reject(error);
  });
}

async function asyncForEach(array: Array<any>, callback: Function): Promise<void> {
  for (let index = 0; index < array.length; index++) {
    await callback(array[index], index, array);
  }
}
