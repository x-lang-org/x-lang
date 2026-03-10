/* @ts-self-types="./x_codegen_wasm.d.ts" */

import * as wasm from "./x_codegen_wasm_bg.wasm";
import { __wbg_set_wasm } from "./x_codegen_wasm_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    XLangCompiler, compile_x_to_js, compile_x_to_ts
} from "./x_codegen_wasm_bg.js";
