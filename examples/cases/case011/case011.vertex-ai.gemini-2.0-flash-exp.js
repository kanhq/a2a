// written by LLM provider: vertex-ai model: gemini-2.0-flash-exp
async function main(config, params) {
  // Define the database connection string.
  const connectionString = config.db;

  // Define the table name.
  const tableName = 'records';

  // Define the file path of the JSON file.
  const filePath = 'datas/records.json';

  // 1. Create table if not exists.
  const createTableQuery = `
    CREATE TABLE IF NOT EXISTS ${tableName} (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT,
      age INTEGER,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )
  `;

  await doAction({
    kind: 'sql',
    connection: connectionString,
    query: createTableQuery,
  });

  // Create index on name column if not exists
  const createIndexQuery = `
    CREATE INDEX IF NOT EXISTS idx_name ON ${tableName} (name)
  `;

  await doAction({
    kind: 'sql',
    connection: connectionString,
    query: createIndexQuery,
  });


  // 2. Read the JSON file.
  const fileContent = await doAction({
    kind: 'file',
    method: 'READ',
    path: filePath,
  });

  const records = fileContent;

  // Prepare the insert query.
  const insertQuery = `
    INSERT INTO ${tableName} (name, age) VALUES (?, ?)
  `;

  // Prepare the data rows for batch insert.
  const batchSize = 100;
  const totalRecords = records.length;

  for (let i = 0; i < totalRecords; i += batchSize) {
    const batch = records.slice(i, i + batchSize);
    const rows = batch.map((record) => [record.name, record.age]);

    // Execute the batch insert.
    await doAction({
      kind: 'sql',
      connection: connectionString,
      query: insertQuery,
      rows: rows,
    });
  }

  return { message: 'Data imported successfully.' };
}
