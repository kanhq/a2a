console.log("test database connection");

console.log(conf.pgsql)

const actions = [
  {
    query: ` 
    CREATE TABLE IF NOT EXISTS a2a_test (
      id INT PRIMARY KEY,
      name TEXT NOT NULL,
      last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );
    `
  },
  {
    query: `
    INSERT INTO a2a_test (id, name) VALUES (?, ?);
    `,
    rows: [[1, "user1"], [2, "user2"]],
  },
  {
    query: `
    SELECT * FROM a2a_test;
    `,
  },
  {
    query: `
    SELECT * FROM a2a_test WHERE id = ?;
    `,
    rows: [1],
  },
  {
    query: `
    DROP TABLE IF EXISTS a2a_test;
    `,
  },
]

for (const action of actions) {
  const sqlAction = {
    ...action,
    kind: 'sql',
    connection: conf.pgsql,
  }
  console.log(action.query.trim())
  const result = doAction(sqlAction)
  console.log(result)
}