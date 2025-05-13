// 假设 config 和 params 已经定义好，并且包含以下内容：
const config = {
    dbconn: {
        // 数据库连接配置
    },
    datasrc: '/path/to/data.csv'
};

const params = {
    // 操作参数
};
async function main(config, params) {
    // Step 1: 创建数据表
    await createTable();

    // Step 2: 插入 CSV 数据
    await insertDataFromCSV(config.datasrc);

    // Step 3: 查询符合条件的数据
    const result = await queryData();
    console.log('查询结果:', result); // 可以在这里添加日志输出

    // Step 4: 删除数据表
    await deleteTable();

    return result;
}

async function createTable() {
    // 使用 doAction 执行创建数据表的 SQL 语句
    const action = {
        kind: 'sql',
        query: `
            CREATE TABLE test_users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                age INTEGER NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
        `,
        connection: config.dbconn.connection
    };
    await doAction(action);
}

async function insertDataFromCSV(filePath) {
    // 读取 CSV 文件并将数据插入到数据表中
    // 这里可以使用 Node.js 的 csv-parser 库或其他方法实现
    const data = await parseCSVFile(filePath);
    const action = {
        kind: 'sql',
        query: `
            INSERT INTO test_users (name, age)
            VALUES (?, ?)
        `,
        connection: config.dbconn.connection,
        rows: data.map(row => [row.name, row.age])
    };
    await doAction(action);
}

async function parseCSVFile(filePath) {
    // 使用 csv-parser 或其他方法解析 CSV 文件
    // 这里仅为示例，实际使用时需要替换为正确的解析逻辑
    const csvData = fs.readFileSync(filePath, 'utf8');
    const parser = new csv.Parser();
    return parser.parse(csvData);
}

async function queryData() {
    // 查询 age 在 40 和 50 之间的记录
    const action = {
        kind: 'sql',
        query: `
            SELECT * FROM test_users WHERE age BETWEEN 40 AND 50;
        `,
        connection: config.dbconn.connection
    };
    const result = await doAction(action);
    return result;
}

async function deleteTable() {
    // 删除数据表
    const action = {
        kind: 'sql',
        query: `
            DROP TABLE test_users;
        `,
        connection: config.dbconn.connection
    };
    await doAction(action);
}
