/** define the type of action  */
type ActionKind =
  | "http"
  | "sql"
  | "file"
  | "email"
  | "shell"
  | "llm"
  | "notify"
  | "enc"
  | "crawl"
  | "web_search"
  ;

/** The base action type, other types will inherit from it */
type BaseAction = {
  kind: ActionKind;
  /** parse will be force use this filed as mimetype instead detected mimetype */
  overrideResultMimeType?: string;
};

/** HttpAction is used to do http request */
type HttpAction = {
  method: "GET" | "POST" | "PUT" | "DELETE" | Uppercase<string>;
  url: string;
  headers?: Record<string, string>;
  body?: any;
  // timeout in seconds
  timeout?: number;
} & BaseAction;

/** HttpAction result */
type HttpResult = {
  /** the status code of the response */
  status: number;
  /** the headers of the response */
  headers: Record<string, string>;
  /** the body of the response
   * 
   *  had been parsed to object by the mimetype detected in headers or the mimetype specified in the action 
   */
  body?: any;
};

/** SqlAction is used to execute a SQL query */
type SqlAction = {
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
type SqlResult = any[];

/** FileAction is used to do operation on local or remote file system */
type FileAction = {
  /** the action to perform
   * - READ : read the file content, the file with well-known mimetype like json, xml, csv, excel, etc will be parsed to object after read
   * - WRITE : write the file content
   * - APPEND : append the file content
   * - LIST : list the file in the directory, the path can have `**` to match all sub directories
   */
  method: "READ" | "WRITE" | "APPEND" | "LIST";
  /** the path/url of the file
   * 
   * it can be a local file path or a remote storage url like s3, blob, aliyun oss, etc.
   *
   * - local file : /path/to/file or file:///path/to/file
   * - aws s3     : s3://bucket/path/to/file
   * - aliyun oss : oss://bucket/path/to/file
   * - azure blob : blob://container/path/to/file
   */
  path: string;
  /** the content to write or append, it will be converted internal to the appropriate format based on the file type
   * so you don't need to worry about the file type, just pass the data you want to write.
   */
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
 * for "LIST" method, the result is a array file info object with the following fields:
 * - 'name' : the file name
 * - 'path' : the file path
 * - 'size' : the file size
 * - 'isDir' : whether it is a directory
 * - 'lastModified' : the last modified time
 */
type FileResult = any;

/** EMailAction is used to send/recv emails*/
type EMailAction = {
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
type EMailMessage = {
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

/** ShellAction used to execute external command*/
type ShellAction = {
  /** the shell command to execute */
  command: string;
  /** the arguments pass to the command */
  args?: string[];
  /** the working directory of the command */
  cwd?: string;
  /** the environment variables of the command */
  env?: Record<string, string>;
} & BaseAction;

type ShellResult = string;

/** LlmAction is used to get result from a Large Language Model, like GPT.
 * 
 * your should build a usefully prompt to the LLM by the user want.
 * when user need generate JSON result, you should set `overrideResultMimeType` to 'application/json' and tell the LLM should generate JSON format result in the system prompt.
 * when user provide any JSON structure description, you should copy it to the system prompt and let the LLM generate the result based on it.
 * when user need process image, you should set the `userImage` field to the image, but don't put any image in the `userPrompt` field.
 */
type LlmAction = {
  /** the connection to the LLM */
  connection: any;
  /** the prompt for 'system' role */
  sysPrompt?: string;
  /** the prompt for 'user' role */
  userPrompt?: string;
  /** the image used in this action */
  userImage?: string;
} & BaseAction;

type LlmResult = any;

/** NotifyAction is used to send message through the IM service 
 *
 * usually there is a webhook url used to send the message.
 */
type NotifyAction = {
  /** the IM service's webhook url */
  url: any;
  /** the message to be sent 
   * message can be string or object,
   * when it is object, it should match the format of the IM service.
   * when it is string, it will be sent as text message type of the IM service, text can be markdown or plain text.
  */
  message?: string | any;
  /** optional title of this message */
  title?: string;
} & BaseAction;

type NotifyResult = any;


type EncMethod =
  'base64' |
  'base64url' |
  'hex' |
  'url' |
  'md5' |
  'sha1' |
  'sha256' |
  'sha1prng' |
  'hmac_md5' |
  'hmac_sha1' |
  'hmac_sha256' |
  'aes_ecb' |
  'aes_cbc'
  ;


/** EncAction is used to do crypto/encoding transform */
type EncAction = {
  /** is this action encrypt/encoding or decrypt/decoding  */
  isDec?: boolean;
  /** chan of encrypt/encoding to perform, you are preferred to combine multiple enc task in one action. */
  methods: EncMethod[];
  /** key used wen do hmac and aes */
  key?: string;
  /** padding method when do AES encrypt/decrypt*/
  padding?: 'zero' | 'space' | 'pkcs5' | 'pkcs7' | 'none';
  /** data used to perform */
  data: string;
} & BaseAction;

type EncResult = string;


type CrawlURL = {
  url: string;
  // the selector to extract the content
  selector?: string;
  // the wait selector to wait for the selector to appear
  wait?: string;
};

/** CrawlAction is used to crawl and extract the web page content
 *
 * the crawl is executed on the headless browser, 
 * the crawled content can be send to a optional llm service to generate structured data.
 * 
 * crawl action should be preferred used to do crawl request, don't use HttpAction to do crawl, because the HttpAction is not able to handle the dynamic content. 
 */
type CrawlAction = {
  // the browser configuration used to crawl
  browser?: any;
  // the urls to crawl, use one crawl action with multiple urls is preferred
  urls: string[] | CrawlURL[];
  // the number of browser to run in parallel
  parallel?: number;
  // the llm connection  used to generate structured data
  llm?: any;
  // a dictionary of fields definition for each url to crawl
  // the key is the url, may contain wildcards
  // the value is the fields to extract, each field must be 'camelCase' english word, you may need do translation for the user's language
  //
  // fields can be configured by the user, or you must build it from on the user's request
  fields?: {
    [url: string]: string[];
  };
} & BaseAction;

// crawl action result is a dictionary of the url and the result
type CrawlResult = any;

/** WebSearchAction is used to search the web 
 * 
 * the search is executed on the headless browser, in the results returned by the search, there are web contents that have already been crawled and can be used directly. see `WebSearchResult` for more details.
 */
type WebSearchAction = {
  // the browser configuration used to search, default is good enough for most cases
  browser?: any;
  // anything to search
  query: string;
  // search engine to use, default is "bing"
  provider: "bing" | "baidu";
  // how many results to return, default is 3
  pages: number;
} & BaseAction;

/**
 * WebSearchResult is the result of the web search
 * 
 * the result is a list of the search result
 */
type WebSearchResult = {
  url: string;
  title: string;
  icon: string;
  body: string;
}[];


export type A2Action = HttpAction | SqlAction | FileAction | EMailAction | ShellAction | LlmAction | NotifyAction | EncAction | CrawlAction | WebSearchAction;

export type ActionResult<T extends A2Action> =
  T extends HttpAction ? HttpResult :
  T extends SqlAction ? SqlResult :
  T extends FileAction ? FileResult :
  T extends EMailAction ? EMailResult :
  T extends ShellAction ? ShellResult :
  T extends LlmAction ? LlmResult :
  T extends NotifyAction ? NotifyResult :
  T extends EncAction ? EncResult :
  T extends CrawlAction ? CrawlResult :
  T extends WebSearchAction ? WebSearchResult :
  never;

// the function to do action 
export declare function doAction<T extends A2Action>(action: T): Promise<ActionResult<T>>;