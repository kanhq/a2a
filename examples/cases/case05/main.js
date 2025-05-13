export async function main(config, params) {
  const readFileAction = {
    kind: "file",
    method: "READ",
    path: params.data01
  };

  const fileResult = await doAction(readFileAction);

  config.data = fileResult
  return config
}
