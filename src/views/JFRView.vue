<template>
  <div class="fixed top-12 left-0 w-12 bottom-0 bg-indigo-100 z-40 border-r border-slate-400 p-2">
    <div class="flex flex-col space-y-2">
      <div class="w-7 h-24 flex-none text-sm border-2 rounded border-slate-400 hover:bg-indigo-300 pt-2">
        <div class="rotate-90">
          <span>threads</span>
        </div>
      </div>
    </div>
  </div>
  <div class="fixed top-12 left-12 h-8 right-0 bg-neutral-100 z-40 border-b border-slate-400 p-2">
    <div class="flex flex-col space-x-2">
    </div>
  </div>
  <div class="fixed h-32 left-12 right-0 bottom-0 bg-neutral-100 z-40 border-t-4 border-slate-400 border-double p-2 overflow-auto">
    <div class="flex flex-col space-x-2 text-sm" v-for="(frame, idx) in selectedFrames" :key="idx">
      {{ frame.typeName }}@{{ frame.methodName }}
    </div>
  </div>
  <div ref="thread-chart-container" class="fixed top-20 left-12 right-0 bottom-32 overflow-auto">
    <div class="flex w-fit" ref="thread-chart"
         @mousemove="chartMouseMove"
         @mouseout="chartMouseOut"
         @click="chartMouseClick()">
      <div ref="thread-chart-header-container"
           class="float-left sticky text-sm left-0 bg-slate-100 border-r-4 border-double border-slate-300">
        <svg ref="thread-chart-header" width="0" height="0"/>
      </div>
      <canvas ref="thread-chart-sample-view"
              class="bg-slate-100"
              width="0"
              height="0"/>
    </div>
    <div class="fixed top-20 left-12 right-0 bottom-32 pointer-events-none">
      <canvas ref="thread-chart-overlay" width="0" height="0" />
    </div>
  </div>
</template>

<script lang="ts">
import { Vue } from 'vue-class-component'
import { Frame, Profile, Sample, StackTrace } from '@/models/jfr/profile'
import { readTextFile } from '@tauri-apps/api/fs'
import { invoke } from '@tauri-apps/api'

interface Thread {
  readonly id: number,
  readonly name: string,
}

interface Dimension {
  readonly width: number,
  readonly height: number,
}

class DateInterval {
  readonly startMillis: number
  readonly endMillis: number
  readonly durationMillis: number

  constructor(startMillis: number, endMillis: number) {
    this.startMillis = startMillis
    this.endMillis = endMillis
    this.durationMillis = endMillis - startMillis
  }
}

interface ThreadChartConfig {
  readonly headerWidth: number,
  readonly fontSize: number,
  readonly borderWidth: number,
  readonly borderColor: string,
  readonly margin: number,
  readonly sampleRenderSize: Dimension,
}

interface ThreadProfile {
  readonly interval: DateInterval,
  // per-thread-id samples
  readonly samples: { [id: number]: Sample[] }
  // max sample num per thread
  readonly maxSampleNum: number
  // thread list sorted by name
  readonly threads: Thread[]
}

const CHART_CONFIG: ThreadChartConfig = {
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

export default class JFRView extends Vue {
  stackTracePool: { [id: number]: StackTrace } = {}
  selectedFrames: Frame[] = []
  selectedThreadIndex = -1
  selectedSampleIndex = -1
  profile?: ThreadProfile

  async mounted() {
    console.log("start")
    const filePath: string = await invoke("jfr_file_path")
    const text = await readTextFile(filePath)
    const profile: Profile = JSON.parse(text)
    console.log("parsed")

    this.stackTracePool = profile.stackTracePool

    const threadProfile = this.convertProfile(profile)
    this.profile = threadProfile
    console.log("converted")

    const header = this.$refs["thread-chart-header"] as HTMLElement
    const sampleView = this.$refs["thread-chart-sample-view"] as HTMLCanvasElement

    const rowHeight = CHART_CONFIG.fontSize + CHART_CONFIG.margin * 2
    const height = rowHeight * threadProfile.threads.length
    const sampleViewWidth = CHART_CONFIG.sampleRenderSize.width * threadProfile.maxSampleNum

    // adjust sizes
    header.setAttribute("width", String(CHART_CONFIG.headerWidth))
    header.setAttribute("height", String(height))
    sampleView.width = sampleViewWidth
    sampleView.height = height

    const ctx = sampleView.getContext("2d", { alpha: false })!
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
      header.appendChild(text)

      if (i < threadProfile.threads.length - 1) {
        const y = yOffset + rowHeight
        const line = document.createElementNS("http://www.w3.org/2000/svg", "line")
        line.setAttribute("x1", String(0))
        line.setAttribute("y1", String(y))
        line.setAttribute("x2", String(CHART_CONFIG.headerWidth))
        line.setAttribute("y2", String(y))
        line.setAttribute("stroke-width", String(CHART_CONFIG.borderWidth))
        line.setAttribute("stroke", String(CHART_CONFIG.borderColor))
        header.appendChild(line)
      }

      const samples = threadProfile.samples[thread.id]
      for (let j = 0; j < samples.length; j++) {
        const sample = samples[j]
        const x = sampleViewWidth * (sample.timestamp - threadProfile.interval.startMillis) /
            threadProfile.interval.durationMillis
        const y = yOffset + (rowHeight - CHART_CONFIG.sampleRenderSize.height) / 2

        const stateName = profile.threadStatePool[sample.threadStateId]
        let fillColor = "#6f6d72"
        if (stateName === "STATE_RUNNABLE") {
          fillColor = "#6cba1e";
        }
        if (stateName === "STATE_SLEEPING") {
          fillColor = "#8d3eee";
        }

        ctx.fillStyle = fillColor
        ctx.fillRect(x, y, CHART_CONFIG.sampleRenderSize.width, CHART_CONFIG.sampleRenderSize.height)
      }
    }

    this.adjustOverlaySize()

    console.log("rendered")
  }

  private convertProfile(profile: Profile): ThreadProfile {
    const perThreadSamples: { [id: number]: Sample[] } = {}

    let startMillis = Infinity
    let endMillis = -1
    let maxSamples = 0

    const threads: Thread[] = []
    profile.samples.forEach(sample => {
      startMillis = Math.min(startMillis, sample.timestamp)
      endMillis = Math.max(endMillis, sample.timestamp)

      let samples = perThreadSamples[sample.threadId]
      if (samples === undefined) {
        threads.push({ id: sample.threadId, name: profile.threadNamePool[sample.threadId] })
        samples = []
      }
      samples.push(sample)
      perThreadSamples[sample.threadId] = samples
      maxSamples = Math.max(maxSamples, samples.length)
    })

    threads.sort((a, b) => a.name.localeCompare(b.name))
    for (let i = 0; i < threads.length; i++) {
      perThreadSamples[threads[i].id].sort((a, b) => a.timestamp - b.timestamp)
    }

    return {
      interval: new DateInterval(startMillis, endMillis),
      samples: perThreadSamples,
      maxSampleNum: maxSamples,
      threads: threads
    }
  }

  private adjustOverlaySize() {
    const overlay = this.$refs["thread-chart-overlay"] as HTMLCanvasElement
    const container = this.$refs["thread-chart-container"] as HTMLElement

    overlay.width = container.clientWidth
    overlay.height = container.clientHeight
  }

  private chartMouseMove(e: MouseEvent) {
    const chart = this.$refs["thread-chart"] as HTMLElement
    const headerContainer = this.$refs["thread-chart-header-container"] as HTMLElement
    const sampleView = this.$refs["thread-chart-sample-view"] as HTMLCanvasElement
    const container = this.$refs["thread-chart-container"] as HTMLElement
    const overlay = this.$refs["thread-chart-overlay"] as HTMLCanvasElement
    const { interval, threads, samples: perThreadSamples } = this.profile!

    const { x } = sampleView.getBoundingClientRect()
    const { y } = chart.getBoundingClientRect()

    const rowHeight = CHART_CONFIG.fontSize + CHART_CONFIG.margin * 2
    const chartX = e.clientX - x
    const chartY = e.clientY - y

    const threadIdx = Math.floor(chartY / rowHeight)
    let sampleIdx = -1

    const highlight: {
      row?: { x: number, y: number, w: number, h: number },
      col?: { x: number, y: number, w: number, h: number },
    } = { row: undefined }

    // Find the sample on the mouse position.
    if (threadIdx >= 0 && threadIdx < threads.length) {
      const absoluteY = threadIdx * rowHeight
      const relativeY = absoluteY - container.scrollTop

      highlight.row = { x: 0, y: relativeY, w: overlay.width, h: rowHeight }

      if (chartX - container.scrollLeft >= 0) {
        const samples = perThreadSamples[threads[threadIdx].id]
        const t = interval.startMillis + ((chartX - CHART_CONFIG.sampleRenderSize.width) / sampleView.width) * interval.durationMillis
        // TODO: bianry search
        for (let i = 0; i < samples.length; i++) {
          const sample = samples[i]
          const sampleX = sampleView.width * (sample.timestamp - interval.startMillis) / interval.durationMillis

          let rightBound = sampleX + CHART_CONFIG.sampleRenderSize.width
          if (i < samples.length - 1) {
            const nextSample = samples[i + 1]
            rightBound = sampleView.width * (nextSample.timestamp - interval.startMillis) / interval.durationMillis
          }
          if (sampleX <= chartX && chartX <= rightBound) {
            sampleIdx = i
            const relativeX = sampleX - container.scrollLeft + headerContainer.offsetWidth
            const sampleY = threadIdx * rowHeight
            const relativeY = sampleY - container.scrollTop
            highlight.col = {
              x: relativeX,
              y: relativeY,
              w: CHART_CONFIG.sampleRenderSize.width,
              h: CHART_CONFIG.fontSize }
            break
          }
        }
      }
    }

    // need re-render
    if (threadIdx !== this.selectedThreadIndex || sampleIdx !== this.selectedSampleIndex) {
      this.selectedThreadIndex = threadIdx
      this.selectedSampleIndex = sampleIdx

      this.clearOverlay(overlay)
      const ctx = overlay.getContext("2d")!

      if (highlight.row !== undefined) {
        ctx.fillStyle = "#40404040"
        ctx.fillRect(highlight.row.x, highlight.row.y, highlight.row.w, highlight.row.h)
      }
      if (highlight.col !== undefined) {
        ctx.fillStyle = "#f04074"
        ctx.fillRect(highlight.col.x, highlight.col.y, highlight.col.w, highlight.col.h)
      }
    }
  }

  private chartMouseOut(e: MouseEvent) {
    this.selectedThreadIndex = -1
    this.clearOverlay(this.$refs["thread-chart-overlay"] as HTMLCanvasElement)
  }

  private chartMouseClick() {
    if (this.selectedThreadIndex >= 0 && this.selectedSampleIndex >= 0) {
      const { threads, samples: perThreadSamples } = this.profile!
      const sample = perThreadSamples[threads[this.selectedThreadIndex].id][this.selectedSampleIndex]
      this.selectedFrames = this.stackTracePool[sample.stackTraceId].frames
    }
  }

  private clearOverlay(overlay: HTMLCanvasElement) {
    const ctx = overlay.getContext("2d")!
    ctx.clearRect(0, 0, overlay.width, overlay.height)
  }
}
</script>
