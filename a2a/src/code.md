You are requested to write some javascript code for use's logic based on the API provided below. You should read the typescript API documentation listed below and write the javascript code accordingly. When you are writing the code, you should to following rules:

- You should define a async function `export async function main(config, params)` which is the entry point of the code. the function should have two parameters:
  - 'config': which is an object that contains the configuration of the application.
  - 'params': which is an object that contains the parameters of the application.
- the `main` function should return the result of the last action.
- You should not use any other libraries, just vanilla javascript.
- Don't use `require` or `import` to include other libs.
- there script will be execute on a custom runtime, there is no `Window`, `Global`, `Buffer` and so on, so you should not use any function provide by that. for example, `btoa`, `atob` etc.
- base64 encoding/decoding is provided by `doAction` with `EncAction`, use it as needed.
- You should use `async/await` for the API calls.
- Read the comments in the API documentation carefully.
- All the API results had parsed to a JSON object.
- You should not use any try/catch block in your code, just let the error throw.
- You should not use any logging mechanism in your code.
- Add comments to your code with same local language as user's language.
- You familiar with the `ffmpeg` command, when user need to do some video/audio processing, you should use the `ShellAction` to call the `ffmpeg` command to do the processing.
- You familiar with the `imagemagick` command, when user need to do some image processing, you should use the `ShellAction` to call the `imagemagick` command to do the processing.

the API documentation is as follows, even though it is in typescript, you should write the code in javascript.

```typescript
export type ActionKind =
  | "http"
  | "sql"
  | "file"
  | "email"
  | "shell"
  | "llm"
  | "notify"
  | "enc";

/** The base action type, all Action had these fields */
export type BaseAction = {
  kind: ActionKind;
  /** parse result will use this filed as mimetype instead detected mimetype */
  overrideResultMimeType?: string;
};

/** Http request information */
export type Http = {
  method: "GET" | "POST" | "PUT" | "DELETE" | Uppercase<string>;
  url: string;
  headers?: Record<string, string>;
  body?: any;
} & BaseAction;

/** Http action result */
export type HttpResult = {
  /** the status code of the response */
  status: number;
  /** the headers of the response */
  headers: Record<string, string>;
  /** the body of the response, had been parsed to object by the mimetype detected in headers or the mimetype specified in the action */
  body: any;
};

/** Execute a SQL query */
export type Sql = {
  /** the connection string of database */
  connection: string;
  /** the SQL to execute,
   *
   * in order to prevent SQL injection, query should use placeholder `?` for the each data to pass
   * be aware that the count of `?` should be equal to the each row of the `rows` field.
   */
  query: string;
  /**
   * the data to pass to the query, the data will be used to replace the placeholder in the query
   *
   * data is 2D array for multiple rows, when do batch insert, update, delete, etc, put all params in this field is preferred instead of multiple query
   */
  rows?: any[][];
} & BaseAction;

/** SQL action result
 * each row is a object with column name as key and column value as value
 */
export type SqlResult = any[];

/** File action */
export type File = {
  // the action to perform
  method: "READ" | "WRITE" | "APPEND";
  /** the path/url of the file
   * it can be a local file path or a remote storage url like s3, blob, aliyun oss, etc.
   *
   * - local file : /path/to/file or file:///path/to/file
   * - aws s3     : s3://bucket/path/to/file
   * - aliyun oss : oss://bucket/path/to/file
   * - azure blob : blob://container/path/to/file
   */
  path: string;
  /** the content to write or append */
  body?: any;
  options?: {
    /** for excel/csv, whether the first row is header name */
    hasHeader?: boolean;
    /** for excel/csv, the column name of the file */
    headers?: string[];
    /** for csv, the delimiter of the file */
    delimiter?: string;
    /** for excel, the sheet name */
    sheet?: string;
  };
} & BaseAction;

/** File action result
 *
 * the result had been parsed to object with the mimetype mapping to the file extension or the mimetype specified in the action.
 */
export type FileResult = any;

/** EMail action */
export type EMailAction = {
  /** the action to perform */
  method: "RECV" | "SEND";
  /** the email account configuration */
  account: any;
  /** the folder when 'RECV' */
  folder?: string;
  /** the previous email id when 'RECV', only id greater then it will be received */
  last_id?: number;
  /** the email to send when 'SEND' */
  message?: any;
};

/** EMail Message */
export type EMailMessage = {
  /** the email id */
  id: number;
  subject: string;
  from: string;
  to: string;
  date: string;
  /** the body of the email */
  body: string;
  /** each attachment is a local file path */
  attachments: string[];
};

type EMailResult = EMailMessage[];

/** Shell action */
export type ShellAction = {
  /** the shell command to execute */
  command: string;
  /** the arguments of the command */
  args?: string[];
  /** the working directory of the command */
  cwd?: string;
  /** the environment variables of the command */
  env?: Record<string, string>;
} & BaseAction;

type ShellResult = string;

/** LLM action
 * this action is used get result from a Large Language Model, like GPT.
 * your should build a usefully prompt to the LLM by the user want.
 * when user need generate JSON result, you should set `overrideResultMimeType` to 'application/json' and tell the LLM should generate JSON format result in the system prompt.
 * when user provide any JSON structure description, you should copy it to the system prompt and let the LLM generate the result based on it.
 * when user need process image, you should set the `userImage` field to the image, but don't put any image in the `userPrompt` field.
 */
export type LlmAction = {
  /** the connection to the LLM */
  connection: any;
  /** the prompt for 'system' role */
  sysPrompt?: string;
  /** the prompt for 'user' role */
  userPrompt?: string;
  /** the image used in this action */
  userImage?: string;
} & BaseAction;

type LLMResult = any;

/** Notify action
 *
 * this action is used to send through the IM service, such as slack, telegram, dingtalk, feishu, wxwork etc.
 * notify action is used to send through the IM service, such as slack, telegram, dingtalk, feishu, wxwork etc.
 * usually there is a webhook url used to send the message.
 * message can be string or object,
 * when it is object, it should match the format of the IM service.
 * when it is string, it will be sent as text message type of the IM service, text can be markdown or plain text.
 */
export type NotifyAction = {
  /** the webhook url */
  url: any;
  /** the message to be sent */
  message?: any;
  /** optional title of this message */
  title?: string;
} & BaseAction;

type NotifyResult = any;

/**
 * Enc action
 *
 * this action is used to do crypto/encoding transform,
 */

export type EncAction = {
  /** is this action encrypt/encoding or decrypt/decoding  */
  isDec?: boolean;
  /** chan of encrypt/encoding to perform, you are preferred to combine multiple enc task in one action.
   * each method can be one of:
   * - base64 : do base64 encoding/decoding
   * - base64url : do base64 url safe encoding/decoding
   * - hex : hex encoding/decoding
   * - url : url encoding/decoding
   * - md5 : calculate the md5 checksum of data
   * - sha1 : calculate the sha1 checksum of data
   * - sha256: calculate the sha256 checksum of data
   * - sha1prng: calculate the sha1prgn checksum of data
   * - hmac_md5: calculate the hmac md5 sign
   * - hmac_sha1: calculate the hmac sha1 sign
   * - hmac_sha256: calculate the hmac sha256 sign
   * - aes_ecb: do AES ECB encrypt/decrypt
   * - aes_cbc: do AES CBC encrypt/decrypt
   */
  methods: string[];
  /** key used wen do hmac and aes */
  key?: string;
  /** padding method when do AES encrypt/decrypt, can be one of
   * - zero: padding with zero
   * - space: padding with space
   * - pkcs5: padding as pkcs5 method
   * - pkcs7: padding as pkcs7 method
   */
  padding?: string;
  /** data used to perform */
  data: string;
} & BaseAction;

type EncResult = string;

/** do http action
 *
 *
 * @param action the action to execute
 * @returns the `body` field of `HttpResult` had been parsed to object by the mimetype detected in the headers or the mimetype specified in the action.
 */
declare function doAction(action: HttpAction): Promise<HttpResult>;

/**
 * do sql action
 *
 * @param action the sql action to perform
 * @returns the result of the action, each row is a object with column name as key and column value as value
 */
declare function doAction(action: SqlAction): Promise<SqlResult>;

/**
 * do file action
 *
 * @param action the file action to perform
 * @returns the result of the action, the result had been parsed by the mimetype detected or the mimetype specified in the action. so usually it the result is not need to be parsed again, just use it directly, below is how the result had been parsed:
 * - 'application/json' : had been parsed to object
 * - 'text/xml'  : had been parsed to object
 * - 'application/yaml' : had been parsed to object
 * - 'text/csv' : had been parsed to a array of object with column name as key and column value as value
 * - 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' : had been parsed to a array of object with column name as key and column value as value
 * - other mimetype : result is the DataURL of the file content
 *
 */
declare function doAction(action: FileAction): Promise<FileResult>;

/**
 * do email action
 *
 * @param action the email action to perform
 * @returns the result of the action, the result had been parsed to object with the following fields:
 * - 'RECV' : each email is a object with the following fields:
 *   - 'id' : the email id
 *   - 'subject' : the email subject
 *   - 'from' : the email from
 *   - 'to' : the email to
 *   - 'date' : the email date
 *   - 'body' : the email body
 *   - 'attachments' : the email attachments, each attachment is a local file path
 * - 'SEND' : the result is empty array
 */
declare function doAction(action: EMailAction): Promise<EMailResult>;

/**
 * do shell action
 *
 * @param action the shell action to perform
 * @returns the result of the action, the result is the stdout of the command
 */
declare function doAction(action: ShellAction): Promise<ShellResult>;

/**
 * do llm action
 *
 * @param action the shell action to perform
 * @returns the result of the action, the result is the stdout of the command
 */
declare function doAction(action: LlmAction): Promise<LlmResult>;

/**
 * do notify action
 *
 *
 * @param action the notify action to perform
 * @returns the result of the action, the result is the stdout of the command
 */
declare function doAction(action: NotifyAction): Promise<NotifyResult>;

/**
 * do enc action
 *
 *
 * @param action the enc action to perform
 * @returns the result of the action, the result is the stdout of the command
 */
declare function doAction(action: EncAction): Promise<EncResult>;
```
