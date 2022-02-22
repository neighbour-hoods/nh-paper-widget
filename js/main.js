// 🤷‍️, from \/
// https://github.com/fengyuanchen/vue-feather/issues/8
import { createApp } from 'vue/dist/vue.esm-bundler';
import { AdminWebsocket, AppWebsocket, InstalledAppInfo } from '@holochain/conductor-api';

const STATUS_INITIAL = 0, STATUS_SAVING = 1, STATUS_SUCCESS = 2, STATUS_FAILED = 3;


const score_comps = [
  { name: "score_comp_const_1",
    comp: "(lam [x] 1)",
  },
  { name: "score_comp_lulz_only",
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
  { name: "score_comp_mbz_only",
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
  { name: "score_comp_lulz_and_mbz_scaled",
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
        [lulz_scalar 5]
        [mbz_scalar 7]
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
    return {
      uploadError: null,
      currentStatus: null,
      hcInfo: null,
      memez: [],
      scoreComps: [],
      selectedScoreCompHash: null,
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
        payload: score_comps[i],
        provenance: cell_id[1],
      });
    }
    console.log("selectedScoreCompHash: ");
    console.log(this.selectedScoreCompHash);

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
    },
    async set_score_comp(scoreCompHash) {
      console.log("set_score_comp: " + scoreCompHash);
      this.selectedScoreCompHash = scoreCompHash;
      this.get_memez()
    },
    count_reaction(aggregated_reactions, reaction_idx) {
      let pair = aggregated_reactions.find(ls => ls[0] == reaction_idx);
      return (pair ? pair[1] : 0)
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
