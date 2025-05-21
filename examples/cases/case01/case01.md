Please use the 'dbconn' connection from the configuration to connect to the database.
Then create a table named 'test_users' with the following fields:

- id: INTEGER, PRIMARY KEY, AUTO_INCREMENT
- name: VARCHAR
- age: INTEGER
- updated_at: TIMESTAMP, DEFAULT CURRENT_TIMESTAMP

Read data from the 'datasrc' data file specified in the configuration, which is a CSV file containing two columns: 'name' and 'age'.

Insert the read data into the 'test_users' table.

Then query the table for records where 'age' is between 40 and 50, and return these as the final result.

Finally, delete the `test_users` table.