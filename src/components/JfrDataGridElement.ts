import {View} from "@finos/perspective";
import { Grid } from "gridjs";
import "gridjs/dist/theme/mermaid.css";
import {TData, TDataArrayRow} from "gridjs/dist/src/types";
import {IPerspectiveViewerElement} from "@finos/perspective-viewer";

export class JfrDataGridElement extends HTMLElement {
    private grid: Grid | undefined = undefined

    constructor() {
        super();
    }

    connectedCallback() {
        // noop
    }

    disconnectedCallback() {
        // noop
    }

    async activate(view: View): Promise<void> {
        if (!this.grid) {
            this.grid = new Grid({
                columns: [],
                sort: true,
                resizable: true,
                fixedHeader: true,
                height: "100%",
                data: [],
                style: {
                    th: {
                        "background-color": "#fdf6e3",
                        "padding": "0.5em",
                    },
                    td: {
                        "background-color": "rgb(245 245 245)",
                        "padding": "0.2em",
                    },
                    container: {
                        "padding": "0.5em",
                        "border-radius": "0",
                        "position": "absolute",
                        "top": "0",
                        "left": "0",
                        "right": "0",
                        "bottom": "0",
                    },
                    header: {
                        "border-radius": "0",
                    },
                    footer: {
                        "border-radius": "0",
                    }
                },
                className: {
                    table: "text-xs",
                    td: "whitespace-pre-wrap"
                }
            }).render(this)
        }
        const table = await (this.parentElement as IPerspectiveViewerElement | null)?.getTable()
        const columns = await table!.columns()

        const json = await view.to_json()
        const data: TDataArrayRow[] = []
        json.forEach((row) => {
            const newRow: TDataArrayRow = []
            columns.forEach((column) => {
                newRow.push(row[column])
            })
            data.push(newRow)
        })
        this.grid.updateConfig({
            columns: columns,
            data: data
        })
        this.grid.forceRender()
    }

    get name() {
        return "Datagrid";
    }

    get category() {
        return "Basic";
    }

    get select_mode() {
        return "toggle";
    }

    get config_column_names() {
        return ["Columns"];
    }

    /**
     * Give the Datagrid a higher priority so it is loaded
     * over the default charts by default.
     */
    get priority() {
        return 1;
    }

    async draw(view: View): Promise<void> {
        return this.activate(view)
    }

    async update(view: View): Promise<void> {
        return this.activate(view)
    }

    async resize() {
        // noop
    }

    async clear() {
        // noop
    }

    save(): string {
        return ""
    }

    restore(token: string) {
        // noop
    }

    async restyle(view: View): Promise<void> {
        // noop
    }

    delete() {
        // noop
    }
}
