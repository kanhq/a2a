

// export declare function doAction(action: any): Promise<any>
// export declare function loadConfig(confDir: string): any

declare module 'a2a' {
  function doAction(action: any): Promise<any>
  function loadConfig(confDir: string): any
  function a2a(confDir: string, codeFile: string, params: any): Promise<any>

  export { doAction, loadConfig, a2a }
}