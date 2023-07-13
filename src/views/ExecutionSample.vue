<template>
  <TabView>
    <!-- Using fixed-position for menubar and absolute-position for chart area -->
    <!-- with tweaking top looks awkward because just stacking with static position -->
    <!-- may be straightforward. However, if we do so, we found that chart area's bottom pane (stacktrace view)'s -->
    <!-- scroll calculation will be broken. -->
    <!-- TODO: To fix, we need to check how splitpane works -->
    <div class="fixed top-12 left-0 right-0 h-12 bg-neutral-100 z-40 border-b border-slate-400 p-2">
      <button class="hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 rounded border-slate-400" @click="open">open file</button>
      <button class="hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 border-slate-500 absolute right-2" @click="loadDemo">load demo</button>
      <span class="h-7 ml-2">thread name:</span>
      <input class="h-7" type="text" placeholder="regex" v-model="threadNameRegex" @change="onFilterChange">
      <span class="h-7 ml-2">stack trace:</span>
      <input class="h-7" type="text" placeholder="match regex" v-model="stackTraceMatchRegex" @change="onFilterChange">
      <span class="h-7 ml-2">&& !</span>
      <input class="h-7" type="text" placeholder="reject regex" v-model="stackTraceRejectRegex" @change="onFilterChange">
      <input v-bind="getInputProps()">
      <button class="disabled:opacity-50 enabled:hover:bg-slate-300 w-8 h-7 ml-2 text-sm text-center border-2 rounded border-slate-400"
              @click="showFlameGraph"
              :disabled="state !== 'loaded'">&#x1f525;</button>
    </div>
    <div class="fixed top-24 left-0 right-0 h-8 bg-neutral-50 z-40 border-b border-slate-400 p-0.5">
      <button class="disabled:opacity-50 enabled:hover:bg-slate-300 w-12 h-5 ml-2 text-xs text-center border-2 border-slate-400"
              @click="onScaleChange(1)"
              :disabled="state !== 'loaded'">reset</button>
      <button class="disabled:opacity-50 enabled:hover:bg-slate-300 w-5 h-5 ml-2 text-xs text-center border-2 border-slate-400"
              @click="onScaleChange(currentScale / 1.5)"
              :disabled="state !== 'loaded'">-</button>
      <button class="disabled:opacity-50 enabled:hover:bg-slate-300 w-5 h-5 ml-2 text-xs text-center border-2 border-slate-400"
              @click="onScaleChange(currentScale * 1.5)"
              :disabled="state !== 'loaded'">+</button>
      <div ref="timeAxis"
           id="time-axis"
           class="absolute z-10 top-0 h-full">
        <span id="time-label" class="text-xs relative hidden" style="transform: translateX(-100%)"/>
      </div>
    </div>
    <canvas ref="headerOverlay"
            id="header-overlay"
            class="fixed pointer-events-none z-10"
            width="0"
            height="0"/>
    <canvas ref="chartOverlay"
            id="chart-overlay"
            class="fixed pointer-events-none z-10"
            width="0"
            height="0"/>
    <div class="absolute top-20 right-0 left-0 bottom-0">
      <splitpanes class="default-theme text-sm" horizontal v-bind="getRootProps()" @resize="syncSize()">
        <pane>
          <div class="w-full h-full">
            <splitpanes vertical @resize="syncSize()">
              <pane size="25"
                    class="overflow-x-hidden overflow-y-auto scrollbar-none relative"
                    @scroll="syncScroll('header')"
                    ref="headerPane"
                    id="header-pane">
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
                    ref="chartPane"
                    id="chart-pane">
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
          <div class="w-full h-full">
            <splitpanes vertical>
              <pane size="75" class="overflow-auto">
                <div class="p-2">
                  <div class="flex flex-col space-x-2 text-sm" v-for="(frame, idx) in highlightedSample?.stackTrace.frames" :key="idx">
                    {{ frame.name }}
                  </div>
                </div>
              </pane>
              <pane>
                <div class="h-full p-2 overflow-auto">
                  <table class="table-auto whitespace-nowrap">
                    <tbody>
                    <tr>
                      <td class="text-right">timestamp :</td>
                      <td>{{ highlightedSample?.timestamp }}</td>
                    </tr>
                    <tr>
                      <td class="text-right">os thread id :</td>
                      <td>{{ highlightedSample?.osThreadId }}</td>
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
    <div class="fixed w-72 h-24 bg-neutral-200 border-neutral-500 p-2 border-2 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
         v-if="state === 'failed'">Failed to load JFR: {{ currentFailure }}</div>
  </TabView>
</template>

<script lang="ts" setup>
import { Splitpanes, Pane } from "splitpanes";
import {
  Renderer,
  ChartConfig, ExecutionSampleInfo,
} from "../../jfrv-wasm/pkg";
import {ComponentPublicInstance, onMounted, onUnmounted, ref} from "vue";
import {FileRejectReason, useDropzone} from "vue3-dropzone";
import 'splitpanes/dist/splitpanes.css';
import TabView from "@/components/TabView.vue";
import {FLAME_GRAPH_CONFIG, FlameGraphWindow} from "@/views/flame-graph";

const CHART_CONFIG: ChartConfig = {
  defaultMargin: 1,
  fontSize: 14, // 0.875rem
  headerConfig: {
    borderWidth: 1,
    borderColorRgbHex: 0x707070,
    elementId: "header",
    paneId: "header-pane",
    overlayElementId: "header-overlay"
  },
  sampleViewConfig: {
    elementId: "thread-chart-sample-view",
    paneId: "chart-pane",
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
    stateUnknownRgbHex: 0x6f6d72,
    stateHiddenRgbHex: 0xc4c4c4,
  },
  overlayConfig: {
    rowHighlightArgbHex: 0x40404040,
    sampleHighlightRgbHex: 0xf04074,
    timestampStrokeWidth: 0.5,
  },
  axisConfig: {
    labelElementId: "time-label",
  }
}

const renderer = ref<Renderer>()

const highlightedSample = ref<ExecutionSampleInfo>()
const headerPane = ref<ComponentPublicInstance>()
const chartPane = ref<ComponentPublicInstance>()
const headerOverlay = ref<HTMLCanvasElement>()
const chartOverlay = ref<HTMLCanvasElement>()
const header = ref<SVGGraphicsElement>()
const chart = ref<HTMLCanvasElement>()
const timeAxis = ref<HTMLElement>()
const threadNameRegex = ref<string>()
const stackTraceMatchRegex = ref<string>()
const stackTraceRejectRegex = ref<string>()
const state = ref<"loading" | "loaded" | "failed">()
const currentFailure = ref<string>()
const currentScale = ref<number>()

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
  accept: [".jfr", ".gz"],
})

onMounted(async () => {
  const wasm = await import("../../jfrv-wasm/pkg")
  renderer.value = new wasm.Renderer(CHART_CONFIG)
  currentScale.value = 1;
  window.addEventListener('resize', syncSize);
})

onUnmounted(() => {
  window.removeEventListener('resize', syncSize);
})

function nullIfEmpty(value: string | undefined): string | null {
  if (value !== undefined && value?.length > 0) {
    return value;
  }
  return null;
}

function onFilterChange() {
  if (!state.value) {
    return
  }

  renderer.value?.apply_filter({
    threadNameRegex: nullIfEmpty(threadNameRegex.value),
    stackTraceMatchRegex: nullIfEmpty(stackTraceMatchRegex.value),
    stackTraceRejectRegex: nullIfEmpty(stackTraceRejectRegex.value),
  })
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
  const file = acceptedFiles[0]
  const buf = await file.arrayBuffer()
  const data = new Uint8Array(buf)

  await loadData(file.name, data)
}

async function loadDemo() {
  state.value = "loading"
  const response = await fetch(`${process.env.BASE_URL}demo.jfr`)
  const buf = await response.arrayBuffer()
  const data = new Uint8Array(buf)

  await loadData("demo.jfr", data)
}

async function onScaleChange(scale: number) {
  let newWidth = CHART_CONFIG.sampleViewConfig.sampleRenderSize.width * scale

  const maxWidth = CHART_CONFIG.sampleViewConfig.sampleRenderSize.width * 8
  const minWidth = 1

  if (newWidth < minWidth || newWidth > maxWidth) {
    return
  }
  currentScale.value = scale
  await renderer.value?.change_scale(newWidth)
}

async function showFlameGraph() {
  const flameGraph = await renderer.value?.flame_graph(FLAME_GRAPH_CONFIG);
  if (!flameGraph) {
    return
  }

  FlameGraphWindow.open(flameGraph)
}

async function loadData(filename: string, data: Uint8Array) {
  threadNameRegex.value = undefined;
  stackTraceMatchRegex.value = undefined;
  stackTraceRejectRegex.value = undefined;
  try {
    const encoding = filename.endsWith(".gz") ? "Gzip" : "Uncompressed"
    renderer.value?.initialize(data, encoding)
    renderer.value?.render()
    syncSize()
  } catch (e: any) {
    state.value = "failed"
    currentFailure.value = e?.toString()
    throw e
  }

  document.title = `jfrv - ${filename}`
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

const syncSize = () => {
  const header = headerOverlay.value
  const chart = chartOverlay.value
  const time = timeAxis.value
  if (!(header && chart && time)) {
    return
  }

  header.width = headerPane.value?.$el.getBoundingClientRect().width
  header.height = headerPane.value?.$el.getBoundingClientRect().height
  header.style.left = `${headerPane.value?.$el.getBoundingClientRect().left}px`
  header.style.top = `${headerPane.value?.$el.getBoundingClientRect().top}px`

  chart.width = chartPane.value?.$el.getBoundingClientRect().width
  chart.height = chartPane.value?.$el.getBoundingClientRect().height
  chart.style.left = `${chartPane.value?.$el.getBoundingClientRect().left}px`
  chart.style.top = `${chartPane.value?.$el.getBoundingClientRect().top}px`

  time.style.width = `${chart.width}px`
  time.style.left = chart.style.left
}
</script>
