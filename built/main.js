"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
// 🤷‍️, from \/
// https://github.com/fengyuanchen/vue-feather/issues/8
var vue_esm_bundler_1 = require("vue/dist/vue.esm-bundler");
var hcClient_1 = require("./hcClient");
var STATUS_INITIAL = 0, STATUS_SAVING = 1, STATUS_SUCCESS = 2, STATUS_FAILED = 3;
var ANN_PATH_STRING = "widget.paperz.annotationz";
var App = {
    name: 'paperz',
    data: function () {
        var hcAppPort = localStorage.getItem('hcAppPort');
        if (hcAppPort === null) {
            hcAppPort = 9999;
            localStorage.setItem('hcAppPort', hcAppPort);
        }
        var hcAdminPort = localStorage.getItem('hcAdminPort');
        if (hcAdminPort === null) {
            hcAdminPort = 9000;
            localStorage.setItem('hcAdminPort', hcAdminPort);
        }
        return {
            hcAppPort: hcAppPort,
            hcAdminPort: hcAdminPort,
            uploadError: null,
            currentStatus: null,
            hcClient: null,
            paperz: [],
            annotationz: [],
            sm_submit: {
                path_string: ANN_PATH_STRING,
                sm_init: {
                    expr_str: "0",
                },
                sm_comp: {
                    expr_str: "(lam [st act]\n  (if (== st 0)\n    (if (== act 0)\n      0\n      (if (== act 1)\n        1\n        st))\n    (if (== st 1)\n      (if (== act 0)\n        0\n        (if (== act 1)\n          1\n          st))\n      st)))",
                },
            },
            sm_init_s: {},
            sm_comp_s: {},
        };
    },
    computed: {
        isInitial: function () {
            return this.currentStatus === STATUS_INITIAL;
        },
        isSaving: function () {
            return this.currentStatus === STATUS_SAVING;
        },
        isSuccess: function () {
            return this.currentStatus === STATUS_SUCCESS;
        },
        isFailed: function () {
            return this.currentStatus === STATUS_FAILED;
        }
    },
    methods: {
        reset: function () {
            // reset form to initial state
            this.currentStatus = STATUS_INITIAL;
            this.uploadError = null;
        },
        handleHcPortSubmit: function () {
            return __awaiter(this, void 0, void 0, function () {
                return __generator(this, function (_a) {
                    localStorage.setItem('hcAppPort', this.hcAppPort);
                    localStorage.setItem('hcAdminPort', this.hcAdminPort);
                    window.location.reload();
                    return [2 /*return*/];
                });
            });
        },
        get_sm_init_and_comp_s: function () {
            return __awaiter(this, void 0, void 0, function () {
                var path_strings, i, path_string, _a, _b, _c, _d;
                return __generator(this, function (_e) {
                    switch (_e.label) {
                        case 0:
                            console.log('get_sm_init_and_comp_s...');
                            path_strings = [ANN_PATH_STRING];
                            i = 0;
                            _e.label = 1;
                        case 1:
                            if (!(i < path_strings.length)) return [3 /*break*/, 5];
                            path_string = path_strings[i];
                            _a = this.sm_init_s;
                            _b = path_string;
                            return [4 /*yield*/, this.hcClient.get_state_machine_init(path_string)];
                        case 2:
                            _a[_b] = _e.sent();
                            console.log("sm_init_s", this.sm_init_s[path_string]);
                            _c = this.sm_comp_s;
                            _d = path_string;
                            return [4 /*yield*/, this.hcClient.get_state_machine_comp(path_string)];
                        case 3:
                            _c[_d] = _e.sent();
                            console.log("sm_comp_s", this.sm_comp_s[path_string]);
                            _e.label = 4;
                        case 4:
                            i++;
                            return [3 /*break*/, 1];
                        case 5:
                            console.log("sm_init_s: ", this.sm_init_s);
                            console.log("sm_comp_s:", this.sm_comp_s);
                            return [2 /*return*/];
                    }
                });
            });
        },
        get_paperz: function () {
            return __awaiter(this, void 0, void 0, function () {
                var _a;
                var _this = this;
                return __generator(this, function (_b) {
                    switch (_b.label) {
                        case 0:
                            console.log("##### BEGIN GETTING PAPERZ #####");
                            _a = this;
                            return [4 /*yield*/, this.hcClient.get_all_paperz()];
                        case 1:
                            _a.paperz = _b.sent();
                            console.log("got all paperz: ", this.paperz);
                            // I think we can turn this into a tree structure using Path on the backend
                            // Will be a bit of legwork to get going but would remove the need for looped callback
                            // patterns like below.
                            // How often will context-resource-sensemaker data be representable by a tree?
                            console.log("Starting 1st async, for each paper, get annotations");
                            return [4 /*yield*/, asyncForEach(this.paperz, function (ele, index) { return __awaiter(_this, void 0, void 0, function () {
                                    var annotationz;
                                    var _this = this;
                                    return __generator(this, function (_a) {
                                        switch (_a.label) {
                                            case 0: return [4 /*yield*/, this.hcClient.get_annotations_for_paper(ele[0])];
                                            case 1:
                                                annotationz = _a.sent();
                                                console.log("Annotationz for paper: ", annotationz);
                                                // for each annotation get all sensemaker data
                                                console.log("Starting 2nd async forEach, get sensemaker");
                                                return [4 /*yield*/, asyncForEach(annotationz, function (ele, index) { return __awaiter(_this, void 0, void 0, function () {
                                                        var sm_data;
                                                        return __generator(this, function (_a) {
                                                            switch (_a.label) {
                                                                case 0:
                                                                    console.log('getting sm_data');
                                                                    return [4 /*yield*/, this.hcClient.get_state_machine_data(ele[0])];
                                                                case 1:
                                                                    sm_data = _a.sent();
                                                                    console.log("sm_data: ", sm_data);
                                                                    annotationz[index].push(sm_data);
                                                                    return [2 /*return*/];
                                                            }
                                                        });
                                                    }); })];
                                            case 2:
                                                _a.sent();
                                                console.log("annotationz: ", annotationz);
                                                this.paperz[index].annotationz = annotationz;
                                                return [2 /*return*/];
                                        }
                                    });
                                }); })];
                        case 2:
                            _b.sent();
                            console.log("paperz: ", this.paperz);
                            console.log("##### END GETTING PAPERZ #####");
                            return [2 /*return*/];
                    }
                });
            });
        },
        // initialize sense maker state machine to
        set_sm_init: function () {
            return __awaiter(this, void 0, void 0, function () {
                var payload, res;
                return __generator(this, function (_a) {
                    switch (_a.label) {
                        case 0:
                            payload = [this.sm_submit.path_string, this.sm_submit.sm_init.expr_str];
                            return [4 /*yield*/, this.hcClient.set_state_machine_init(payload)];
                        case 1:
                            res = _a.sent();
                            console.log("set_sm_init res: ", res);
                            this.get_sm_init_and_comp_s();
                            return [2 /*return*/];
                    }
                });
            });
        },
        set_sm_comp: function () {
            return __awaiter(this, void 0, void 0, function () {
                var payload, res;
                return __generator(this, function (_a) {
                    switch (_a.label) {
                        case 0:
                            payload = [this.sm_submit.path_string, this.sm_submit.sm_comp.expr_str];
                            return [4 /*yield*/, this.hcClient.set_state_machine_comp(payload)];
                        case 1:
                            res = _a.sent();
                            console.log("set_sm_comp res: ", res);
                            this.get_sm_init_and_comp_s();
                            return [2 /*return*/];
                    }
                });
            });
        },
        handlePaperSubmit: function (evt) {
            return __awaiter(this, void 0, void 0, function () {
                var file, obj, hh;
                var _a;
                return __generator(this, function (_b) {
                    switch (_b.label) {
                        case 0:
                            this.currentStatus = STATUS_SAVING;
                            console.log("handlePaperSubmit: ", evt);
                            file = evt.target.files[0];
                            _a = {
                                filename: file.name
                            };
                            return [4 /*yield*/, getBase64(file)];
                        case 1:
                            obj = (_a.blob_str = _b.sent(),
                                _a);
                            console.log(obj);
                            return [4 /*yield*/, this.hcClient.upload_paper(obj)];
                        case 2:
                            hh = _b.sent();
                            console.log('Paper HeaderHash: ', hh);
                            this.currentStatus = STATUS_INITIAL;
                            return [4 /*yield*/, this.get_paperz()];
                        case 3:
                            _b.sent();
                            return [2 /*return*/];
                    }
                });
            });
        },
        handleCreateAnnotationSubmit: function (paper_ref, evt) {
            return __awaiter(this, void 0, void 0, function () {
                var obj, _a, eh, hh;
                return __generator(this, function (_b) {
                    switch (_b.label) {
                        case 0:
                            obj = {
                                paper_ref: paper_ref,
                                page_num: evt.target.elements.page_num.valueAsNumber,
                                paragraph_num: evt.target.elements.paragraph_num.valueAsNumber,
                                what_it_says: evt.target.elements.what_it_says.value,
                                what_it_should_say: evt.target.elements.what_it_should_say.value,
                            };
                            console.log("handleCreateAnnotationSubmit: obj: ", obj);
                            return [4 /*yield*/, this.hcClient.create_annotation(obj)];
                        case 1:
                            _a = _b.sent(), eh = _a[0], hh = _a[1];
                            console.log("handleCreateAnnotationSubmit:");
                            console.log(eh);
                            console.log(hh);
                            return [4 /*yield*/, this.get_paperz()];
                        case 2:
                            _b.sent();
                            return [2 /*return*/];
                    }
                });
            });
        },
        handleStepSm: function (ann_eh, evt) {
            return __awaiter(this, void 0, void 0, function () {
                var obj;
                return __generator(this, function (_a) {
                    switch (_a.label) {
                        case 0:
                            console.log("handleStepSm:");
                            console.log(ann_eh);
                            console.log(evt);
                            obj = [
                                ANN_PATH_STRING,
                                ann_eh,
                                evt.target.elements.action.value
                            ];
                            console.log(obj);
                            return [4 /*yield*/, this.hcClient.step_sm(obj)];
                        case 1:
                            _a.sent();
                            return [4 /*yield*/, this.get_paperz()];
                        case 2:
                            _a.sent();
                            return [2 /*return*/];
                    }
                });
            });
        }
    },
    ////////////////////////////////////////////////////////////////////////////////
    // lifecycle hooks
    ////////////////////////////////////////////////////////////////////////////////
    beforeMount: function () {
        return __awaiter(this, void 0, void 0, function () {
            var _a, admin, cells, installed_app_id, sensemakerDnaHash, installedApp, startApp1, sensemakerCell, res;
            return __generator(this, function (_b) {
                switch (_b.label) {
                    case 0:
                        console.log('beforeMount');
                        _a = this;
                        return [4 /*yield*/, hcClient_1.HcClient.initialize(this.hcAppPort, this.hcAdminPort)];
                    case 1:
                        _a.hcClient = _b.sent();
                        console.log('hcClient: ', this.hcClient);
                        admin = this.hcClient.adminWs;
                        return [4 /*yield*/, admin.listCellIds()];
                    case 2:
                        cells = _b.sent();
                        console.log('cells: ', cells);
                        installed_app_id = 'sensemaker';
                        if (!(cells.length == 1)) return [3 /*break*/, 7];
                        console.log('cells == 1');
                        return [4 /*yield*/, admin.registerDna({
                                path: './result/social_sensemaker.dna',
                            })];
                    case 3:
                        sensemakerDnaHash = _b.sent();
                        return [4 /*yield*/, admin.installApp({
                                installed_app_id: installed_app_id,
                                agent_key: this.hcClient.cellId[1],
                                dnas: [{ hash: sensemakerDnaHash, role_id: 'thedna' }],
                            })];
                    case 4:
                        installedApp = _b.sent();
                        console.log('installedApp: ', installedApp);
                        return [4 /*yield*/, admin.enableApp({ installed_app_id: installed_app_id })];
                    case 5:
                        startApp1 = _b.sent();
                        console.log('startApp1: ', startApp1);
                        sensemakerCell = installedApp.cell_data[0].cell_id;
                        console.log('setting sensemakerCell: ', sensemakerCell);
                        return [4 /*yield*/, this.hcClient.set_sensemaker_cell_id(sensemakerCell)];
                    case 6:
                        res = _b.sent();
                        console.log('set_sensemaker_cell_id: ', res);
                        _b.label = 7;
                    case 7: return [4 /*yield*/, this.get_sm_init_and_comp_s()];
                    case 8:
                        _b.sent();
                        return [4 /*yield*/, this.get_paperz()];
                    case 9:
                        _b.sent();
                        return [2 /*return*/];
                }
            });
        });
    },
    mounted: function () {
        this.reset();
    },
};
var app = (0, vue_esm_bundler_1.createApp)(App);
app.mount('#app');
////////////////////////////////////////////////////////////////////////////////
// helpers
////////////////////////////////////////////////////////////////////////////////
function getBase64(file) {
    return new Promise(function (resolve, reject) {
        var reader = new FileReader();
        reader.readAsDataURL(file);
        reader.onload = function () { return resolve(reader.result); };
        reader.onerror = function (error) { return reject(error); };
    });
}
function asyncForEach(array, callback) {
    return __awaiter(this, void 0, void 0, function () {
        var index;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    index = 0;
                    _a.label = 1;
                case 1:
                    if (!(index < array.length)) return [3 /*break*/, 4];
                    return [4 /*yield*/, callback(array[index], index, array)];
                case 2:
                    _a.sent();
                    _a.label = 3;
                case 3:
                    index++;
                    return [3 /*break*/, 1];
                case 4: return [2 /*return*/];
            }
        });
    });
}
