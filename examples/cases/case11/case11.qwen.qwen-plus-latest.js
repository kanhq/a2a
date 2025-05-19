// written by LLM provider: qwen model: qwen-plus-latest
async function main(config, params) {
  // Read the JSON file using FileAction with READ method
  const readAction = {
    kind: 'file',
    method: 'READ',
    path: 'data.json'
  };

  const jsonData = await doAction(readAction); // Read the content of data.json

  // Write the JSON data into an Excel (xlsx) file using FileAction with WRITE method
  const writeAction = {
    kind: 'file',
    method: 'WRITE',
    path: 'data.xlsx',
    body: jsonData,
    options: {
      sheet: 'Sheet1' // Specify the sheet name in the Excel file
    }
  };

  const result = await doAction(writeAction); // Write data to data.xlsx
  return result; // Return the result of the last action
}
