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
    <div class="flex flex-col space-x-2">
    </div>
  </div>
  <div class="fixed top-20 left-12 right-0 bottom-32 overflow-auto">
    <div class="flex w-fit">
      <div class="sticky text-sm left-0 bg-slate-100 border-r-4 border-double border-slate-300 min-h-full">
        <svg ref="thread-chart-header" width="0" height="0"/>
      </div>
      <canvas ref="thread-chart-sample-view"
              class="bg-slate-100"
              width="0"
              height="0"/>
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
  threadStatePool: { [id: number]: string } = {}
  selectedFrames: Frame[] = []

  async mounted() {
    console.log("start")
    const filePath: string = await invoke("jfr_file_path")
    const text = await readTextFile(filePath)
    const profile: Profile = JSON.parse(text)
    console.log("parsed")

    this.stackTracePool = profile.stackTracePool
    this.threadStatePool = profile.threadStatePool

    const threadProfile = this.convertProfile(profile)
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

        const stateName = this.threadStatePool[sample.threadStateId]
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

    return {
      interval: new DateInterval(startMillis, endMillis),
      samples: perThreadSamples,
      maxSampleNum: maxSamples,
      threads: threads
    }
  }
}
</script>
