import * as duckdb from "@duckdb/duckdb-wasm";
import duckdb_wasm from "@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm";
import * as arrow from "apache-arrow";
import {RecordBatchReader} from "apache-arrow";

const BUNDLE = {
    mvp: {
        mainModule: duckdb_wasm,
        mainWorker: new URL('@duckdb/duckdb-wasm/dist/duckdb-browser-mvp.worker.js', import.meta.url).toString()
    }
}

export class DB {
    private db: duckdb.AsyncDuckDB;
    private cid: number | undefined;
    constructor() {
        const dbWorker = new Worker(BUNDLE.mvp.mainWorker)
        const logger = new duckdb.ConsoleLogger(duckdb.LogLevel.WARNING)
        this.db = new duckdb.AsyncDuckDB(logger, dbWorker)
    }

    async init() {
        await this.db.instantiate(BUNDLE.mvp.mainModule, BUNDLE.mvp.mainWorker)
        this.cid = await this.db.connectInternal()
    }

    async query(sql: string): Promise<Uint8Array> {
        if (this.cid === undefined) {
            await this.init()
        }
        return await this.db.runQuery(this.cid!, sql)
    }

    async registerFile(filename: string, data: Uint8Array): Promise<void> {
        await this.db.dropFile(filename)
        return this.db.registerFileBuffer(filename, data)
    }

    async schema(): Promise<Schema> {
        const result: Schema = { tables: [] }
        const tableNames = new arrow.Table(RecordBatchReader
            .from((await this.query("show tables")).buffer)).toArray();
        for (let i = 0; i < tableNames.length; i++){
            const tableName = tableNames[i]["name"]
            const table: Table = { name: tableName, columns: [] }
            new arrow.Table(RecordBatchReader.from((await this.query(`pragma show('"${tableName}"')`)).buffer))
                .toArray().forEach((row2) => {
                const columnName = row2["column_name"]
                const columnType = row2["column_type"]
                table.columns.push({ name: columnName, type: columnType })
            })
            result.tables.push(table)
        }
        return result
    }

    getFile(name: string): Promise<Uint8Array> {
        return this.db.copyFileToBuffer(name)
    }
}

export interface Schema {
    tables: Table[];
}

export interface Table {
    name: string;
    columns: Column[];
}

export interface Column {
    name: string;
    type: string;
}
