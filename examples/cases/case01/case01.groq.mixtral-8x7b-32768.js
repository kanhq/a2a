/**
 * The entry point of the code
 * @param config - the configuration of the application
 * @param params - the parameters of the application
 * @returns the result of the last action
 */
async function main(config, params) {
  // Create the SQL action to create the test_users table
  const createTableAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `
      CREATE TABLE IF NOT EXISTS test_users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT,
        age INTEGER,
