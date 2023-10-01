import * as duckdb from "@duckdb/duckdb-wasm";
import duckdb_wasm from "@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm";

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

    async registerFile(file: File): Promise<void> {
        await this.db.dropFile(file.name)
        return this.db.registerFileHandle(file.name, file, duckdb.DuckDBDataProtocol.BROWSER_FILEREADER, true)
    }

    getFile(name: string): Promise<Uint8Array> {
        return this.db.copyFileToBuffer(name)
    }
}
