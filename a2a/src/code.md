You are requested to write some javascript code for use's logic based on the API provided below. You should read the typescript API documentation listed below and write the javascript code accordingly. When you are writing the code, you should to following rules:

- You should define a async function `async function main(config, params)` which is the entry point of the code. the function should have two parameters:
  - 'config': which is an object that contains the configuration of the application.
  - 'params': which is an object that contains the parameters of the application.
- the `main` function should return the result of the last action.
- You should not use any other libraries, just vanilla javascript.
- You should use `async/await` for the API calls.
- Read the comments in the API documentation carefully.
- All the API results had parsed to a JSON object.
- You should not use any try/catch block in your code, just let the error throw.
- You should not use any logging mechanism in your code.
- Add comments to your code with same local language as user's language.

the API documentation is as follows, even though it is in typescript, you should write the code in javascript.

```typescript
export type ActionKind = "http" | "sql" | "file";

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
} & BaseAction;

/** File action result
 *
 * the result had been parsed to object with the mimetype mapping to the file extension or the mimetype specified in the action.
 */
export type FileResult = any;

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
 *
 */
declare function doAction(action: FileAction): Promise<FileResult>;
```
