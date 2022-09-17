<template>
  <div class="fixed top-12 left-0 h-10 right-0 bg-neutral-100 z-40 border-b border-slate-400 p-2">
    <button class="hover:bg-slate-300 w-24 h-6 text-sm text-center border-2 rounded border-slate-400" @click="open">open file</button>
    <input v-bind="getInputProps()">
    <div class="flex flex-col space-x-2">
    </div>
  </div>
  <splitpanes class="default-theme absolute top-10 text-sm" horizontal v-bind="getRootProps()">
    <pane>
      <div class="relative w-full h-full">
        <splitpanes vertical>
          <pane size="25"
                class="overflow-x-hidden overflow-y-auto scrollbar-none relative"
                @scroll="syncScroll('header')"
                ref="headerPane">
            <canvas ref="header-overlay"
                    id="header-overlay"
                    class="absolute top-0 left-0 pointer-events-none"
                    width="0"
                    height="0"/>
            <svg ref="header"
                 id="header"
                 class="absolute top-0 left-0"
                 @mousemove="onHeaderMouseMove"
                 @mouseout="onMouseOut"
                 width="0"
                 height="0"/>
          </pane>
          <pane class="overflow-auto relative"
                @scroll="syncScroll('chart')"
                ref="chartPane">
            <canvas ref="chart-overlay"
                    id="chart-overlay"
                    class="absolute top-0 left-0 pointer-events-none"
                    width="0"
                    height="0"/>
            <canvas ref="chart"
                    id="thread-chart-sample-view"
                    @mousemove="onChartMouseMove"
                    @mouseout="onMouseOut"
                    @click="onChartClick"
                    class="bg-slate-100"
                    width="0"
                    height="0"/>
            <div v-if="!fileLoaded">
              <p v-if="isDragActive">Drop here ...</p>
              <p v-else>Drag & drop OR press "open file" to select JFR file</p>
            </div>
          </pane>
        </splitpanes>
      </div>
    </pane>
    <pane size="40" class="overflow-auto">
      <div class="flex flex-col space-x-2 text-sm" v-for="(frame, idx) in highlightedFrames" :key="idx">
        {{ frame.typeName }}@{{ frame.methodName }}
      </div>
    </pane>
  </splitpanes>
</template>

<script lang="ts" setup>
import { Splitpanes, Pane } from "splitpanes";
import {
  JfrRenderer,
  Frame,
  ChartConfig,
} from "../../pete2-wasm/pkg";
import {ComponentPublicInstance, onMounted, ref} from "vue";
import {FileRejectReason, useDropzone} from "vue3-dropzone";
import 'splitpanes/dist/splitpanes.css';

const CHART_CONFIG: ChartConfig = {
  headerWidth: 360,
  fontSize: 14, // 0.875rem
  borderWidth: 1,
  borderColor: "#707070",
  margin: 1,
  sampleRenderSize: {
    width: 6,
    height: 8,
  }
}

const renderer = ref<JfrRenderer>()
const fileLoaded = ref(false)

const highlightedFrames = ref<Frame[]>()
const headerPane = ref<ComponentPublicInstance>()
const chartPane = ref<ComponentPublicInstance>()
const header = ref<SVGGraphicsElement>()
const chart = ref<HTMLCanvasElement>()

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
  accept: ".jfr",
})

onMounted(async () => {
  const wasm = await import("../../pete2-wasm/pkg")
  renderer.value = new wasm.JfrRenderer()
})

function onChartClick(e: MouseEvent) {
  const stackTrace = renderer.value?.on_chart_click(
      e.clientX - chart.value!.getBoundingClientRect().x,
      e.clientY - chart.value!.getBoundingClientRect().y)
  console.log("stack trace: " + stackTrace)
  highlightedFrames.value = stackTrace?.frames
}

function onHeaderMouseMove(e: MouseEvent) {
  renderer.value?.on_header_mouse_move(
      e.clientX - header.value!.getBoundingClientRect().x,
      e.clientY - header.value!.getBoundingClientRect().y)
}

function onChartMouseMove(e: MouseEvent) {
  renderer.value?.on_chart_mouse_move(
      e.clientX - chart.value!.getBoundingClientRect().x,
      e.clientY - chart.value!.getBoundingClientRect().y)
}

function onMouseOut() {
  renderer.value?.on_mouse_out()
}

async function openFile(acceptedFiles: File[], rejectReasons: FileRejectReason[]) {
  const buf = await acceptedFiles[0].arrayBuffer()
  const data = new Uint8Array(buf)

  renderer.value?.load_jfr(data, CHART_CONFIG)
  renderer.value?.render()

  fileLoaded.value = true
}

const syncScroll = (src: "header" | "chart") => {
  const header = headerPane.value?.$el
  const chart = chartPane.value?.$el

  if (src === "header") {
    chart.scrollTop = header.scrollTop
  }
  if (src === "chart") {
    header.scrollTop = chart.scrollTop
  }
}
</script>
