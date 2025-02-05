

// export declare function doAction(action: any): Promise<any>
// export declare function loadConfig(confDir: string): any

import { A2Action, ActionResult } from "./action";

declare function doAction<T extends A2Action>(action: T): Promise<ActionResult<T>>;
declare function loadConfig(confDir: string): any
declare function a2a(confDir: string, codeFile: string, params: any): Promise<any>

export { doAction, loadConfig, a2a, A2Action, ActionResult }