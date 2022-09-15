<template>
  <div class="fixed top-12 left-0 h-10 right-0 bg-neutral-100 z-40 border-b border-slate-400 p-2">
    <button class="hover:bg-slate-300 w-24 h-6 text-sm text-center border-2 rounded border-slate-400" @click="open">open file</button>
    <input v-bind="getInputProps()">
    <div class="flex flex-col space-x-2">
    </div>
  </div>
  <splitpanes class="default-theme absolute top-10 text-sm" horizontal v-bind="getRootProps()">
    <pane>
      <splitpanes vertical>
        <pane size="25"
              class="overflow-x-hidden overflow-y-auto scrollbar-none"
              @scroll="syncScroll('header')"
              ref="headerPane">
          <svg ref="header" />
        </pane>
        <pane class="overflow-auto"
              @scroll="syncScroll('chart')"
              ref="chartPane">
          <canvas ref="thread-chart-sample-view"
                  id="thread-chart-sample-view"
                  class="bg-slate-100" width="0" height="0" />
          <div v-if="!fileLoaded">
            <p v-if="isDragActive">Drop here ...</p>
            <p v-else>Drag & drop OR press "open file" to select JFR file</p>
          </div>
        </pane>
      </splitpanes>
    </pane>
    <pane size="40">
      <div class="flex flex-col space-x-2 text-sm" v-for="(frame, idx) in selectedFrames" :key="idx">
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
  Profile,
  Sample,
  StackTrace,
  ChartConfig,
  ThreadProfile, SampledThread,
} from "../../pete2-wasm/pkg";
import {ComponentPublicInstance, onMounted, ref} from "vue";
import {FileRejectReason, useDropzone} from "vue3-dropzone";

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

const stackTracePool = ref<Record<number, StackTrace>>()
const selectedFrames = ref<Frame[]>()
const selectedThreadIndex = ref(-1)
const selectedSampleIndex = ref(-1)
const profile = ref<ThreadProfile>()
const renderer = ref<JfrRenderer>()
const fileLoaded = ref(false)

const headerPane = ref<ComponentPublicInstance>()
const chartPane = ref<ComponentPublicInstance>()
const header = ref<SVGGraphicsElement>()

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

function calculateHeaderWidth(threads: SampledThread[]): number {
  let text = ""
  threads.forEach(t => {
    if (text.length < t.name.length) {
      text = t.name
    }
  })
  const textElement = document.createElementNS("http://www.w3.org/2000/svg", "text")
  const textNode = document.createTextNode(text)
  textElement.appendChild(textNode)

  return textElement.getBBox().width
}

async function openFile(acceptedFiles: File[], rejectReasons: FileRejectReason[]) {
  const buf = await acceptedFiles[0].arrayBuffer()
  const data = new Uint8Array(buf)

  const threadProfile: ThreadProfile = renderer.value!.load_jfr(data, CHART_CONFIG)!
  stackTracePool.value = threadProfile.stackTracePool
  profile.value = threadProfile
  console.log("converted")

  const rowHeight = CHART_CONFIG.fontSize + CHART_CONFIG.margin * 2
  const height = rowHeight * threadProfile.threads.length

  for (let i = 0; i < threadProfile.threads.length; i++) {
    const thread = threadProfile.threads[i]
    const yOffset = rowHeight * i;

    const text = document.createElementNS("http://www.w3.org/2000/svg", "text")
    const textNode = document.createTextNode(thread.name)
    text.setAttribute("x", String(CHART_CONFIG.margin))
    // y is the baseline of the text.
    // so we add fontSize to the current offset.
    // also add margin to allocate the margin-top.
    text.setAttribute("y", String(yOffset + CHART_CONFIG.fontSize + CHART_CONFIG.margin))
    text.appendChild(textNode)
    header.value?.appendChild(text)
  }
  fileLoaded.value = true

  // adjust sizes then render borders
  const headerWidth = header.value?.getBBox().width

  for (let i = 0; i < threadProfile.threads.length; i++) {
    const yOffset = rowHeight * i;
    if (i < threadProfile.threads.length - 1) {
      const y = yOffset + rowHeight
      const line = document.createElementNS("http://www.w3.org/2000/svg", "line")
      line.setAttribute("x1", String(0))
      line.setAttribute("y1", String(y))
      line.setAttribute("x2", String(headerWidth))
      line.setAttribute("y2", String(y))
      line.setAttribute("stroke-width", String(CHART_CONFIG.borderWidth))
      line.setAttribute("stroke", String(CHART_CONFIG.borderColor))
      header.value?.appendChild(line)
    }
  }

  header.value?.setAttribute("width", String(headerWidth))
  header.value?.setAttribute("height", String(height))
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
