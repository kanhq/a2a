// written by LLM provider: chuanshen model: rendu-2.5
async function main(config, params) {
    const dbconn = config.dbconn; 
    const datasrc = config.datasrc;
    
    // Create 'test_users' table if not exists
    await doAction({
        kind: 'sql',
        options: {
            command: `CREATE TABLE IF NOT EXISTS test_users (
                id INT AUTO_INCREMENT PRIMARY KEY,
                name VARCHAR(255),
                age INT,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );`
        }
    });

    // Load data from 'datasrc' CSV file
    const data = await doAction({
        kind: 'file',
        options: {
            headers: ['name', 'age'],
            delimiter: ',' 
        },
        body: datasrc
    });

    // Insert data into 'test_users' table
    const insertPromises = data.map(record => doAction({
        kind: 'sql',
        body: `INSERT INTO test_users (name, age) VALUES (?, ?);`,
        options: {
            params: [record.name, record.age]
        }
    }));
    await Promise.all(insertPromises);

    // Query 'age' between 40 and 50
    const result = await doAction({
        kind: 'sql',
        body: 'SELECT * FROM test_users WHERE age BETWEEN 40 AND 50;'
    });

    // Drop 'test_users' table
    await doAction({
        kind: 'sql',
        body: 'DROP TABLE test_users;'
    });

    return result;
}
