<template>
  <!-- Using fixed-position for menubar and absolute-position for chart area -->
  <!-- with tweaking top looks awkward because just stacking with static position -->
  <!-- may be straightforward. However, if we do so, we found that chart area's bottom pane (stacktrace view)'s -->
  <!-- scroll calculation will be broken. -->
  <!-- TODO: To fix, we need to check how splitpane works -->
  <div class="fixed top-12 left-0 right-0 h-12 bg-neutral-100 z-40 border-b border-slate-400 p-2">
    <button class="hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 rounded border-slate-400" @click="open">open file</button>
    <span class="h-7 ml-2">thread name:</span>
    <input class="h-7" type="text" placeholder="regex" v-model="filterRegex" @change="onFilterChange">
    <input v-bind="getInputProps()">
    <div class="flex flex-col space-x-2">
    </div>
  </div>
  <div class="absolute top-12 right-0 left-0 bottom-0">
    <splitpanes class="default-theme text-sm" horizontal v-bind="getRootProps()">
      <pane>
        <div class="w-full h-full">
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
              <div v-if="!state">
                <p v-if="isDragActive">Drop here ...</p>
                <p v-else>Drag & drop OR press "open file" to select JFR file</p>
              </div>
            </pane>
          </splitpanes>
        </div>
      </pane>
      <pane size="40" class="overflow-auto">
        <div>
          <div class="flex flex-col space-x-2 text-sm" v-for="(frame, idx) in highlightedFrames" :key="idx">
            {{ frame.typeName }}@{{ frame.methodName }}
          </div>
        </div>
      </pane>
    </splitpanes>
  </div>
  <div class="fixed w-72 h-24 bg-neutral-200 border-neutral-500 p-2 border-2 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
       v-if="state === 'loading'">Loading...</div>
</template>

<script lang="ts" setup>
import { Splitpanes, Pane } from "splitpanes";
import {
  Renderer,
  ChartConfig, StackFrame,
} from "../../pete2-wasm/pkg";
import {ComponentPublicInstance, onMounted, ref} from "vue";
import {FileRejectReason, useDropzone} from "vue3-dropzone";
import 'splitpanes/dist/splitpanes.css';

const CHART_CONFIG: ChartConfig = {
  defaultMargin: 1,
  fontSize: 14, // 0.875rem
  headerConfig: {
    borderWidth: 1,
    borderColorRgbHex: 0x707070,
    elementId: "header",
    overlayElementId: "header-overlay"
  },
  sampleViewConfig: {
    elementId: "thread-chart-sample-view",
    overlayElementId: "chart-overlay",
    sampleRenderSize: {
      width: 6,
      height: 8
    },
    backgroundRgbHex: 0xf5f5f5
  },
  threadStateColorConfig: {
    stateRunnableRgbHex: 0x6cba1e,
    stateSleepingRgbHex: 0x8554c2,
    stateUnknownRgbHex: 0x6f6d72
  },
  overlayConfig: {
    rowHighlightArgbHex: 0x40404040,
    sampleHighlightRgbHex: 0xf04074
  }
}

const renderer = ref<Renderer>()

const highlightedFrames = ref<StackFrame[]>()
const headerPane = ref<ComponentPublicInstance>()
const chartPane = ref<ComponentPublicInstance>()
const header = ref<SVGGraphicsElement>()
const chart = ref<HTMLCanvasElement>()
const filterRegex = ref<string>()
const state = ref<"loading" | "loaded">()

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
  renderer.value = new wasm.Renderer(CHART_CONFIG)
})

function onFilterChange() {
  if (!state.value) {
    return
  }

  if (filterRegex.value !== undefined && filterRegex.value?.length > 0) {
    renderer.value?.apply_filter({
      threadNameRegex: filterRegex.value
    })
  } else {
    renderer.value?.apply_filter({
      threadNameRegex: null
    })
  }
}

function onChartClick() {
  const stackTrace = renderer.value?.on_chart_click()
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
  state.value = "loading"
  const buf = await acceptedFiles[0].arrayBuffer()
  const data = new Uint8Array(buf)

  filterRegex.value = undefined;
  try {
    renderer.value?.initialize(data)
    renderer.value?.render()
  } catch (e) {
    state.value = undefined
    throw e
  }

  state.value = "loaded"
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
