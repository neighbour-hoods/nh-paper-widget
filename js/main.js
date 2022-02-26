// 🤷‍️, from \/
// https://github.com/fengyuanchen/vue-feather/issues/8
import { createApp } from 'vue/dist/vue.esm-bundler';
import { AdminWebsocket, AppWebsocket, InstalledAppInfo } from '@holochain/conductor-api';

const STATUS_INITIAL = 0, STATUS_SAVING = 1, STATUS_SUCCESS = 2, STATUS_FAILED = 3;


const score_comps = [
  { name: "equal value",
    comp: `
(let ([foldl
         (fix (lam [foldl]
           (lam [f acc xs]
             (if (null xs)
               acc
               (foldl
                 f
                 (f acc (head xs))
                 (tail xs))))))]
        [lulz_scalar 1]
        [mbz_scalar 1]
        [folder
         (lam [acc tup]
           (if (== 0 (fst tup))
               (pair (+ (* lulz_scalar (snd tup))
                        (fst acc))
                     (snd acc))
               (if (== 1 (fst tup))
                   (pair (fst acc)
                         (+ (* mbz_scalar (snd tup))
                            (snd acc)))
                   acc)))])
    (lam [vals]
      (let ([res (foldl folder (pair 0 0) vals)])
        (+ (fst res)
           (snd res)))))
`,
  },
  { name: "😂 only",
    comp:`
(let ([foldl
       (fix (lam [foldl]
         (lam [f acc xs]
           (if (null xs)
             acc
             (foldl
               f
               (f acc (head xs))
               (tail xs))))))]
      [folder
       (lam [acc tup]
         (if (== 0 (fst tup))
             (+ (snd tup) acc)
             acc))])
  (lam [vals]
    (foldl folder 0 vals)))
`,
  },
  { name: "🤯 only",
    comp: `
(let ([foldl
       (fix (lam [foldl]
         (lam [f acc xs]
           (if (null xs)
             acc
             (foldl
               f
               (f acc (head xs))
               (tail xs))))))]
      [folder
       (lam [acc tup]
         (if (== 1 (fst tup))
             (+ (snd tup) acc)
             acc))])
  (lam [vals]
    (foldl folder 0 vals)))
`,
  },
  { name: "double 🤯",
    comp: `
(let ([foldl
         (fix (lam [foldl]
           (lam [f acc xs]
             (if (null xs)
               acc
               (foldl
                 f
                 (f acc (head xs))
                 (tail xs))))))]
        [lulz_scalar 1]
        [mbz_scalar 2]
        [folder
         (lam [acc tup]
           (if (== 0 (fst tup))
               (pair (+ (* lulz_scalar (snd tup))
                        (fst acc))
                     (snd acc))
               (if (== 1 (fst tup))
                   (pair (fst acc)
                         (+ (* mbz_scalar (snd tup))
                            (snd acc)))
                   acc)))])
    (lam [vals]
      (let ([res (foldl folder (pair 0 0) vals)])
        (+ (fst res)
           (snd res)))))
`,
  }
];

const App = {
  name: 'app',
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
      memez: [],
      scoreComps: [],
      selectedScoreCompHash: null,
      scoreCompCreateForm: {
        name: "my cool score comp",
        comp: "(lam [x] 1)",
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

    for (let i = 0; i < score_comps.length; i++) {
      await this.create_score_comp(score_comps[i]);
    }

    await this.get_score_comps();

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
        payload: data,
        provenance: cell_id[1],
      });
      this.currentStatus = STATUS_INITIAL;
      // TODO handle `res` error cases

      this.get_memez()
    },
    async get_memez() {
      if (this.selectedScoreCompHash) {
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
          payload: this.selectedScoreCompHash,
          provenance: cell_id[1],
        });
        console.log("all meme strings: ");
        console.log(res);
        this.memez = res.sort(function(a, b) { return b.opt_score - a.opt_score });
      } else {
        console.log("no selectedScoreCompHash")
      }
    },
    async react(meme_eh, reaction_name) {
      console.log("react: " + meme_eh + " | " + reaction_name);
      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;
      let flag = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'memez_main_zome',
        fn_name: 'react_to_meme',
        payload: {
          meme_eh: meme_eh,
          reaction_name: reaction_name,
          count: 1
        },
        provenance: cell_id[1],
      });
      console.log("react_to_meme success flag: " + flag);
      this.get_memez();
    },
    async set_score_comp(scoreCompHash) {
      console.log("set_score_comp: " + scoreCompHash);
      this.selectedScoreCompHash = scoreCompHash;
      this.get_memez()
    },
    count_reaction(aggregated_reactions, reaction_idx) {
      let pair = aggregated_reactions.find(ls => ls[0] == reaction_idx);
      return (pair ? pair[1] : 0)
    },
    async create_score_comp(score_comp) {
      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;

      for (let i = 0; i < score_comps.length; i++) {
        this.selectedScoreCompHash = await this.hcInfo.appWs.callZome({
          cap: null,
          cell_id: cell_id,
          zome_name: 'memez_main_zome',
          fn_name: 'create_score_computation',
          payload: score_comp,
          provenance: cell_id[1],
        });
      }
      console.log("selectedScoreCompHash: ");
      console.log(this.selectedScoreCompHash);
    },
    async get_score_comps() {
      let info = await this.hcInfo.appWs.appInfo({
        // TODO figure out why this works... it shouldn't, I think?
        installed_app_id: 'test-app',
      });
      const cell_id = info.cell_data[0].cell_id;

      this.scoreComps = await this.hcInfo.appWs.callZome({
        cap: null,
        cell_id: cell_id,
        zome_name: 'memez_main_zome',
        fn_name: 'get_score_computations',
        payload: null,
        provenance: cell_id[1],
      });
      console.log("scoreComps: ");
      console.log(this.scoreComps);
    },
    async handleScoreCompSubmit() {
      this.create_score_comp(this.scoreCompCreateForm);

      await this.get_score_comps();
    },
    async handleHcPortSubmit() {
      localStorage.setItem('hcAppPort', this.hcAppPort);
      localStorage.setItem('hcAdminPort', this.hcAdminPort);
      window.location.reload()
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
