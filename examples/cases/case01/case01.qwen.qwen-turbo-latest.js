// 定义主函数入口
export async function main(config, params) {
    const dbConnection = config.dbconn; // 获取数据库连接字符串
    const csvFilePath = config.datasrc;  // 获取CSV文件路径

    // 创建数据表 'test_users'
    await doAction({
        kind: "sql",
        connection: dbConnection,
        query: `CREATE TABLE IF NOT EXISTS test_users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(255),
            age INT,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )`,
    });

    // 从CSV文件读取数据
    const fileData = await doAction({
        kind: "file",
        path: csvFilePath,
        method: "READ"
    });

    // 插入数据到 'test_users' 表中
    const insertQueries = fileData.map(row => ({
        kind: "sql",
        connection: dbConnection,
        query: `INSERT INTO test_users (name, age) VALUES (?, ?)`,
        rows: [[row.name, row.age]]
    }));

    for (const query of insertQueries) {
        await doAction(query);
    }

    // 查询年龄在40到50之间的用户记录
    const result = await doAction({
        kind: "sql",
        connection: dbConnection,
        query: `SELECT * FROM test_users WHERE age BETWEEN 40 AND 50`
    });

    // 删除 'test_users' 表
    await doAction({
        kind: "sql",
        connection: dbConnection,
        query: `DROP TABLE test_users`
    });

    // 返回查询结果
    return result;
}
