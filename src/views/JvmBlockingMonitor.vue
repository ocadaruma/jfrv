<template>
  <!-- Using fixed-position for menubar and absolute-position for chart area -->
  <!-- with tweaking top looks awkward because just stacking with static position -->
  <!-- may be straightforward. However, if we do so, we found that chart area's bottom pane (stacktrace view)'s -->
  <!-- scroll calculation will be broken. -->
  <!-- TODO: To fix, we need to check how splitpane works -->
  <div class="fixed top-12 left-0 right-0 h-12 bg-neutral-100 z-40 border-b border-slate-400 p-2">
    <button class="hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 rounded border-slate-400" @click="open">open file</button>
    <button class="hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 border-slate-500 absolute right-2" @click="loadDemo">load demo</button>
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
                      id="jbm-header-overlay"
                      class="absolute top-0 left-0 pointer-events-none"
                      width="0"
                      height="0"/>
              <svg ref="header"
                   id="jbm-header"
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
                      id="jbm-chart-overlay"
                      class="absolute top-0 left-0 pointer-events-none"
                      width="0"
                      height="0"/>
              <canvas ref="chart"
                      id="jbm-thread-chart-sample-view"
                      @mousemove="onChartMouseMove"
                      @mouseout="onMouseOut"
                      @click="onChartClick"
                      class="bg-slate-100"
                      width="0"
                      height="0"/>
              <div v-if="!state">
                <p v-if="isDragActive">Drop here ...</p>
                <p v-else>Drag & drop OR press "open file" to select jbm log file</p>
              </div>
            </pane>
          </splitpanes>
        </div>
      </pane>
      <pane size="40">
        <div class="w-full h-full">
          <splitpanes vertical>
            <pane size="75" class="overflow-auto">
              <div class="p-2">
                <div class="flex flex-col space-x-2 text-sm"
                     v-for="(frame, idx) in highlightedSample?.stackTrace?.frames"
                     :key="idx">
                  {{ frame.methodName }}
                </div>
              </div>
            </pane>
            <pane>
              <div class="h-full p-2 overflow-auto">
                <table class="table-auto whitespace-nowrap">
                  <tbody>
                  <tr>
                    <td class="text-right">thread :</td>
                    <td>{{ highlightedSample?.threadName }}</td>
                  </tr>
                  <tr>
                    <td class="text-right">offcpu start :</td>
                    <td>{{ highlightedSample?.offcpuStart }}</td>
                  </tr>
                  <tr>
                    <td class="text-right">offcpu end :</td>
                    <td>{{ highlightedSample?.offcpuEnd }}</td>
                  </tr>
                  <tr>
                    <td class="text-right">duration (ms) :</td>
                    <td>{{ highlightedSample?.durationMillis }}</td>
                  </tr>
                  </tbody>
                </table>
              </div>
            </pane>
          </splitpanes>
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
  JbmRenderer,
  JbmChartConfig, StackFrame, JbmSampleInfo,
} from "../../jfrv-wasm/pkg";
import {ComponentPublicInstance, onMounted, ref} from "vue";
import {FileRejectReason, useDropzone} from "vue3-dropzone";
import 'splitpanes/dist/splitpanes.css';

const CHART_CONFIG: JbmChartConfig = {
  defaultMargin: 1,
  fontSize: 14, // 0.875rem
  headerConfig: {
    borderWidth: 1,
    borderColorRgbHex: 0x707070,
    elementId: "jbm-header",
    overlayElementId: "jbm-header-overlay"
  },
  sampleViewConfig: {
    elementId: "jbm-thread-chart-sample-view",
    overlayElementId: "jbm-chart-overlay",
    sampleRenderHeight: 8,
    sampleWidthPerHour: 256,
    backgroundRgbHex: 0xf5f5f5
  },
  threadStateColorConfig: {
    stateRunnableRgbHex: 0x6cba1e,
    stateSleepingRgbHex: 0x8554c2,
    stateUnknownRgbHex: 0x6f6d72
  },
  overlayConfig: {
    rowHighlightArgbHex: 0x40404040,
    sampleHighlightRgbHex: 0xf04074,
  }
}

const renderer = ref<JbmRenderer>()

const highlightedSample = ref<JbmSampleInfo>()
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
})

onMounted(async () => {
  const wasm = await import("../../jfrv-wasm/pkg")
  renderer.value = new wasm.JbmRenderer(CHART_CONFIG)
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
  highlightedSample.value = renderer.value?.on_chart_click()
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

  await loadData(data)
}

async function loadDemo() {
  state.value = "loading"
  const response = await fetch(`${process.env.BASE_URL}jbm.log`)
  const buf = await response.arrayBuffer()
  const data = new Uint8Array(buf)

  await loadData(data)
}

async function loadData(data: Uint8Array) {
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
