import { doAction } from '../index'
import { expect, test } from "bun:test";


test('sync function from native code', async () => {
  console.log("test database connection");

  const conf = {
    "pgsql": "postgresql://jia:lijia@127.0.0.1:5432/kaccount?sslmode=disable",
    "mysql": "mysql://lijia:123456@127.0.0.1:3306/lijia",
    "sqlite": "sqlite://a2a_test.sqlite3"
  }

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
    const result = await doAction(sqlAction)
    console.log(result)
  }
})
