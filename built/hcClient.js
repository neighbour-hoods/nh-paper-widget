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
exports.HcClient = void 0;
var client_1 = require("@holochain/client");
var HcClient = /** @class */ (function () {
    function HcClient(adminWs, appWs, cellId) {
        this.adminWs = adminWs;
        this.appWs = appWs;
        this.cellId = cellId;
    }
    HcClient.initialize = function (appPort, adminPort) {
        return __awaiter(this, void 0, void 0, function () {
            var appWs, adminWs, info, cellId;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, client_1.AppWebsocket.connect('ws://localhost:' + appPort.toString())];
                    case 1:
                        appWs = _a.sent();
                        return [4 /*yield*/, client_1.AdminWebsocket.connect('ws://localhost:' + adminPort.toString())];
                    case 2:
                        adminWs = _a.sent();
                        return [4 /*yield*/, appWs.appInfo({
                                installed_app_id: 'test-app',
                            })];
                    case 3:
                        info = _a.sent();
                        console.log('info: ', info);
                        cellId = info.cell_data[0].cell_id;
                        return [2 /*return*/, new HcClient(adminWs, appWs, cellId)];
                }
            });
        });
    };
    HcClient.prototype.callZome = function (fn_name, payload) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.appWs.callZome({
                            cap: null,
                            cell_id: this.cellId,
                            zome_name: 'paperz_main_zome',
                            fn_name: fn_name,
                            payload: payload,
                            provenance: this.cellId[1],
                        })];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    HcClient.prototype.set_sensemaker_cell_id = function (payload) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        console.log('set_sensemaker_cell_id: payload: ', payload);
                        return [4 /*yield*/, this.callZome('set_sensemaker_cell_id', payload)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    HcClient.prototype.get_sensemaker_cell_id = function () {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('get_sensemaker_cell_id', null)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    /// Plain holochain widget calls
    HcClient.prototype.get_all_paperz = function () {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('get_all_paperz', null)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    HcClient.prototype.get_annotations_for_paper = function (ele) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('get_annotations_for_paper', ele)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    HcClient.prototype.upload_paper = function (payload) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('upload_paper', payload)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    // Holochain call with sensemaker bridge call
    HcClient.prototype.create_annotation = function (payload) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('create_annotation', payload)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    // Sensemaker bridge calls
    HcClient.prototype.get_state_machine_init = function (payload) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('get_state_machine_init', payload)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    ;
    HcClient.prototype.get_state_machine_comp = function (payload) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('get_state_machine_comp', payload)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    HcClient.prototype.get_state_machine_data = function (payload) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('get_state_machine_data', payload)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    HcClient.prototype.set_state_machine_comp = function (payload) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('set_state_machine_comp', payload)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    HcClient.prototype.set_state_machine_init = function (payload) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('set_state_machine_init', payload)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    HcClient.prototype.step_sm = function (payload) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, this.callZome('step_sm_remote', payload)];
                    case 1: return [2 /*return*/, _a.sent()];
                }
            });
        });
    };
    return HcClient;
}());
exports.HcClient = HcClient;
