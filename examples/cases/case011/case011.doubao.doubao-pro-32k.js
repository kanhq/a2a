// written by LLM provider: doubao model: doubao-pro-32k
// Entry point function
async function main(config, params) {
    // Step 1: Create the 'records' table and unique index if not exist
    const createTableAction = {
        kind: "sql",
        connection: config.db,
        query: `
            CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT,
                age INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_name ON records(name);
        `
    };
    await doAction(createTableAction);

    // Step 2: Read the 'records.json' file
    const readFileAction = {
        kind: "file",
        method: "READ",
        path: "datas/records.json"
    };
    const records = await doAction(readFileAction);

    // Step 3: Insert or update records in batches of 5
    const batchSize = 5;
    for (let i = 0; i < records.length; i += batchSize) {
        const batch = records.slice(i, i + batchSize);
        const rows = batch.map(record => [record.name, record.age]);
        const upsertAction = {
            kind: "sql",
            connection: config.db,
            query: `
                INSERT OR REPLACE INTO records (name, age)
                VALUES (?,?);
            `,
            rows: rows
        };
        await doAction(upsertAction);
    }

    // Return the result of the last action (although upsert doesn't usually return meaningful data in this case)
    return await doAction({
        kind: "sql",
        connection: config.db,
        query: "SELECT COUNT(*) FROM records"
    });
}


