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
              :disabled="state !== 'loaded'">Run query</button>
      <button class="hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 border-slate-500 absolute right-2" @click="loadDemo">load demo</button>
      <input v-bind="getInputProps()">
      <div class="flex flex-col space-x-2">
      </div>
    </div>
    <div class="absolute top-12 right-0 left-0 bottom-0">
      <splitpanes class="default-theme text-sm" horizontal v-bind="getRootProps()">
        <pane>
          <div class="w-full h-full">
            <codemirror
              v-model="query"
              placeholder="Enter query..."
              :extensions="[sql()]"/>
          </div>
        </pane>
        <pane size="50">
          <div class="w-full h-full">
            <perspective-viewer class="w-full h-full" ref="queryViewer" id="query-viewer"></perspective-viewer>
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
import {
  JbmRenderer,
  JbmChartConfig, JbmSampleInfo,
} from "../../jfrv-wasm/pkg";
import {ComponentPublicInstance, onMounted, ref} from "vue";
import {FileRejectReason, useDropzone} from "vue3-dropzone";
import 'splitpanes/dist/splitpanes.css';
import TabView from "@/components/TabView.vue";
import perspective from '@finos/perspective';
import {PerspectiveViewerElement} from "@finos/perspective-viewer/dist/esm/perspective";
import {Codemirror} from "vue-codemirror";
import {sql} from "@codemirror/lang-sql";
import {DB} from "@/views/duckdb";

const state = ref<"loading" | "loaded" | "failed" | "executing">()
const queryViewer = ref<PerspectiveViewerElement>()
const query = ref<string>("")
const db = ref<DB>()

const {
  getRootProps,
  getInputProps,
  isDragActive,
  open
} = useDropzone({
  onDrop: openFile,
  multiple: false,
  noClick: true,
  noKeyboard: true,
  accept: [".jfr"],
})

onMounted(async () => {
  db.value = new DB()
  await db.value?.init()
})

async function runQuery() {
  state.value = "executing"
  const result = await db.value!.query(query.value).catch((e) => {
    state.value = "failed"
    throw e
  })
  state.value = "loaded"
  const worker = perspective.worker()
  const table = await worker.table(result.buffer)
  await queryViewer.value?.load(table)
}

async function openFile(acceptedFiles: File[], rejectReasons: FileRejectReason[]) {
  state.value = "loading"
  const file = acceptedFiles[0]
  await loadData(file)
}

async function loadDemo() {
  // state.value = "loading"
  // const response = await fetch(`${process.env.BASE_URL}demo.jfr`)
  // const buf = await response.arrayBuffer()
  // const data = new Uint8Array(buf)
  //
  // await loadData("demo.jfr", data)
}

async function loadData(file: File) {
  await db.value?.registerFile(file).catch((e) => {
    state.value = "failed"
    throw e
  })
  await db.value?.query(`call jfr_attach('${file.name}')`).catch((e) => {
    state.value = "failed"
    throw e
  })
  state.value = "loaded"
}
</script>
