export async function main(config, params) {
    // 使用配置中的 'dbconn' 连接数据库
    const dbConnection = config.dbconn;

    // 创建 'test_users' 数据表
    await doAction({
        kind: "sql",
        connection: dbConnection,
        query: `
            CREATE TABLE test_users (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255),
                age INT,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        `
    });

    // 从配置的 'datasrc' 数据文件读取数据
    const csvData = await doAction({
        kind: "file",
        method: "READ",
        path: config.datasrc
    });

    // 将读取到的数据插入到 'test_users' 表中
    const rowsToInsert = csvData.map(row => [row.name, row.age]);

    await doAction({
        kind: "sql",
        connection: dbConnection,
        query: `
            INSERT INTO test_users (name, age) VALUES (?, ?)
        `,
        rows: rowsToInsert
    });

    // 查询 'age' 在 40 和 50 之间的记录
    const results = await doAction({
        kind: "sql",
        connection: dbConnection,
        query: `
            SELECT * FROM test_users WHERE age BETWEEN 40 AND 50
        `
    });

    // 删除 'test_users' 数据表
    await doAction({
        kind: "sql",
        connection: dbConnection,
        query: `
            DROP TABLE test_users
        `
    });

    // 返回结果
    return results;
}
