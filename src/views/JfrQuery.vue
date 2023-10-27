<template>
  <TabView>
    <!-- Using fixed-position for menubar and absolute-position for chart area -->
    <!-- with tweaking top looks awkward because just stacking with static position -->
    <!-- may be straightforward. However, if we do so, we found that chart area's bottom pane (stacktrace view)'s -->
    <!-- scroll calculation will be broken. -->
    <!-- TODO: To fix, we need to check how splitpane works -->
    <div class="fixed top-12 left-0 right-0 h-12 bg-neutral-100 z-40 border-b border-slate-400 p-2">
      <button class="hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 rounded border-slate-400" @click="open">open file</button>
      <button class="disabled:opacity-50 enabled:hover:bg-slate-300 w-24 h-7 ml-2 text-sm text-center border-2 rounded border-slate-400"
              @click="runQuery"
              :disabled="state === 'executing'">Run query</button>
      <span class="h-7 ml-2 text-xs ">file name: {{ currentFile }}</span>
      <button class="hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 border-slate-500 absolute right-2" @click="loadDemo">load demo</button>
      <input v-bind="getInputProps()">
      <div class="flex flex-col space-x-2">
      </div>
    </div>
    <div class="absolute top-12 right-0 left-0 bottom-0">
      <splitpanes class="default-theme text-sm" horizontal v-bind="getRootProps()">
        <pane>
          <splitpanes vertical>
            <pane size="20" class="overflow-auto">
              <div class="m-2 text-xs">
                <TreeNode :data="dbSchema.tables" />
              </div>
            </pane>
            <pane>
              <div class="w-full h-full overflow-auto">
                <codemirror
                    v-model="query"
                    :placeholder='`Enter query...  (Drag & drop OR press "open file" to attach jfr file)`'
                    :extensions="[
                        sql({dialect: sqlDialect()}),
                        keymap.of([{ key: 'Ctrl-Enter', run: () => { runQuery(); return true; } }])
                        ]"/>
              </div>
            </pane>
          </splitpanes>
        </pane>
        <pane size="50">
          <div class="w-full h-full overflow-auto">
            <div v-if="state === 'failed'"
                 class="w-full h-full text-xs p-2 text-red-500">
              <pre>
{{ currentError }}
              </pre>
            </div>
            <perspective-viewer
                v-else
                class="w-full h-full"
                ref="queryViewer"
                id="query-viewer"></perspective-viewer>
          </div>
        </pane>
      </splitpanes>
    </div>
    <div class="fixed w-72 h-24 bg-neutral-200 border-neutral-500 p-2 border-2 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
         v-if="state === 'loading'">Loading...</div>
    <div class="fixed w-72 h-24 bg-neutral-200 border-neutral-500 p-2 border-2 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
         v-if="state === 'executing'">Executing...</div>
  </TabView>
</template>

<script lang="ts" setup>
import { Splitpanes, Pane } from "splitpanes";
import {onMounted, ref} from "vue";
import {FileRejectReason, useDropzone} from "vue3-dropzone";
import 'splitpanes/dist/splitpanes.css';
import TabView from "@/components/TabView.vue";
import TreeNode from "@/components/TreeNode.vue";
import perspective from '@finos/perspective';
import {PerspectiveViewerElement} from "@finos/perspective-viewer/dist/esm/perspective";
import {Codemirror} from "vue-codemirror";
import {sql, SQLDialect, SQLite} from "@codemirror/lang-sql";
import {keymap} from "@codemirror/view";
import {DB, Schema} from "@/views/duckdb";
import {RecordBatchReader, Table as ArrowTable} from "apache-arrow";
import {JfrDataGridElement} from "@/components/JfrDataGridElement";
import {format} from "date-fns";
const state = ref<"loading" | "loaded" | "failed" | "executing">()
const currentError = ref<string>("")
const currentFile = ref<string>("")
const queryViewer = ref<PerspectiveViewerElement>()
const query = ref<string>("")
const db = ref<DB>()
const dbSchema = ref<Schema>({ tables: [] })
// const completionSchema = computed(() => {
//   const schema: { [table: string]: string[] } = {}
//   dbSchema.value.tables.forEach((table) => {
//     schema[`"${table.name}"`] = table.columns.map((column) => column.name)
//   })
//   return schema
// })

interface TypeDescriptor {
  tableType: string
  converter: (value: any) => any
}

const {
  getRootProps,
  getInputProps,
  open
} = useDropzone({
  onDrop: openFile,
  multiple: false,
  noClick: true,
  noKeyboard: true,
  accept: [".jfr", ".gz"],
})

customElements.define(
    "jfr-datagrid",
    JfrDataGridElement)
customElements.whenDefined("jfr-datagrid")
    .then(() => {
      customElements.get("perspective-viewer")
          .registerPlugin("jfr-datagrid")
    })

function sqlDialect(): SQLDialect {
  return SQLDialect.define({
    ...SQLite.spec,
    keywords: SQLite.spec.keywords?.concat(" jfr_attach", " jfr_scan"),
  })
}

onMounted(async () => {
  db.value = new DB()
  await db.value?.init()
})

async function runQuery() {
  state.value = "executing"
  const result = await db.value!.query(query.value).catch((e) => {
    state.value = "failed"
    currentError.value = e.toString()
    throw e
  })
  state.value = "loaded"
  dbSchema.value = await db.value!.schema()
  const worker = perspective.worker()
  const reader = RecordBatchReader.from(result.buffer)

  const schema: { [key: string]: TypeDescriptor } = {}
  const arrowTable = new ArrowTable(reader)

  arrowTable.schema.fields.forEach((field) => {
    try {
      let tableType = "string"
      let converter = (value: any) => value.toString()

      const arrowType = field.type.toString()
      if (arrowType === "Bool") {
        tableType = "boolean"
        converter = (value: any) => value
      } else if (arrowType.startsWith("Float")) {
        tableType = "float"
        converter = (value: any) => value
      } else if (arrowType.startsWith("Int")) {
        tableType = "integer"
        converter = (value: any) => value
      } else if (arrowType.startsWith("Timestamp")) {
        // We just format timestamp as string for now
        // We consider to support timestamp if it is needed in the future
        tableType = "string"
        converter = (value: any) => format(value, "yyyy-MM-dd HH:mm:ss.SSS")
      } else if (arrowType === "Date") {
        tableType = "string"
        converter = (value: any) => format(value, "yyyy-MM-dd")
      }
      schema[field.name] = { tableType, converter }
    } catch (e: any) {
      state.value = "failed"
      currentError.value = e.toString()
      throw e
    }
  })
  const data: Array<Record<string, Array<string | boolean | number>>> = []
  arrowTable.toArray().forEach((arrowRow) => {
    const row: Record<string, Array<string | boolean | number>> = {}
    Object.entries(schema).forEach(([key, typeDesc]) => {
      const val = arrowRow[key]
      row[key] = [typeDesc.converter(val)]
    })
    data.push(row)
  })

  const tableSchema: { [key: string]: string } = {}
  Object.entries(schema).forEach(([key, typeDesc]) => {
    tableSchema[key] = typeDesc.tableType
  })
  const table = await worker.table(tableSchema)

  if (arrowTable.numRows > 0) {
    table.update(data)
  }

  await queryViewer.value?.load(table)
  queryViewer.value?.reset()
}

async function openFile(acceptedFiles: File[], rejectReasons: FileRejectReason[]) {
  state.value = "loading"
  const file = acceptedFiles[0]
  const buf = await file.arrayBuffer()
  await loadData(file.name, new Uint8Array(buf))
}

async function loadDemo() {
  state.value = "loading"
  const response = await fetch(`${process.env.BASE_URL}demo.jfr`)
  const buf = await response.arrayBuffer()
  const data = new Uint8Array(buf)

  await loadData("demo.jfr", data)
}

async function loadData(filename: string, data: Uint8Array) {
  await db.value?.registerFile(filename, data).catch((e) => {
    state.value = "failed"
    currentError.value = e.toString()
    throw e
  })
  await db.value?.query(`call jfr_attach('${filename}')`).catch((e) => {
    state.value = "failed"
    currentError.value = e.toString()
    throw e
  })
  state.value = "loaded"
  currentFile.value = filename
  dbSchema.value = await db.value!.schema()
}
</script>
