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
      <span class="h-7 ml-2 text-xs">file name: {{ currentFile }}</span>
      <button class="hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 border-slate-500 absolute right-2" @click="loadDemo">load demo</button>
      <input v-bind="getInputProps()">
      <div class="flex flex-col space-x-2">
      </div>
    </div>
    <div class="absolute top-12 right-0 left-0 bottom-0">
      <splitpanes class="default-theme text-sm" horizontal v-bind="getRootProps()">
        <pane>
          <div class="w-full h-full overflow-auto">
            <codemirror
              v-model="query"
              :placeholder='`Enter query...\n\nDrag & drop OR press "open file" to attach jfr file`'
              :extensions="[sql(), keymap.of([{ key: 'Ctrl-Enter', run: () => { runQuery(); return true; } }])]"/>
          </div>
        </pane>
        <pane size="50">
          <div class="w-full h-full">
            <div v-if="state === 'failed'"
                 class="w-full h-full text-sm p-2 text-red-500">
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
import perspective from '@finos/perspective';
import {PerspectiveViewerElement} from "@finos/perspective-viewer/dist/esm/perspective";
import {Codemirror} from "vue-codemirror";
import {sql} from "@codemirror/lang-sql";
import {keymap} from "@codemirror/view";
import {DB} from "@/views/duckdb";
import {RecordBatchReader, Table, tableFromJSON} from "apache-arrow";
import {valueToString} from "apache-arrow/util/pretty";

const state = ref<"loading" | "loaded" | "failed" | "executing">()
const currentError = ref<string>("")
const currentFile = ref<string>("")
const queryViewer = ref<PerspectiveViewerElement>()
const query = ref<string>("")
const db = ref<DB>()

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
  const worker = perspective.worker()
  const reader = RecordBatchReader.from(result.buffer)

  const typeMap = (key: string): string => {
    if (key === "Bool") return "boolean";
    if (key === "Date") return "date";
    if (key.startsWith("Float")) return "float";
    if (key.startsWith("Int")) return "integer";
    if (key.startsWith("Timestamp")) return "datetime";
    if (key.startsWith("Utf8")) return "string";

    throw new Error(`Unsupported type: ${key}`)
  }
  const schema: { [key: string]: string } = {}
  const arrowTable = new Table(reader)

  arrowTable.schema.fields.forEach((field) => {
    try {
      schema[field.name] = typeMap(field.type.toString())
    } catch (e: any) {
      state.value = "failed"
      currentError.value = e.toString()
      throw e
    }
  })

  const table = await worker.table(schema)
  if (arrowTable.numRows > 0) {
    table.update(result.buffer)
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
}
</script>
