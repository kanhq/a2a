There is a 'records.json' in 'datas' folder. You need to import the data into the database.

use `config.db` as the database connection.

1. Create a new table named 'records' if not exist in the database. The table should have the following columns:

- id: integer, primary key, auto increment
- name: string
- age: integer
- created_at: datetime, default to current timestamp
  Create index on 'name' column if not exist.

2. Read the 'records.json' file and insert the data into the 'records' table in batch. The batch size should be 100.

- 'records.json' is a list of records, each record is a dictionary with 'name' and 'age' keys.
