<template>
  <div>
    <div v-for="thread in threads" :key="thread.id" class="h-4 text-sm">
      <div class="flex w-fit">
        <div class="flex-none
                    sticky
                    left-0
                    w-96
                    border-r border-r-slate-500
                    border-t border-t-slate-300
                    bg-slate-50">
          <div class="truncate">{{ thread.name }}</div>
        </div>
        <div class="grow border-t border-t-slate-300">
          <svg :width="chartArea().w">
            <rect v-for="sample in perThreadSamples.get(thread.id)"
                  :key="sample.timestamp"
                  :x="chartArea().w * (sample.timestamp - profileDuration.startMillis) / (profileDuration.endMillis - profileDuration.startMillis)"
                  y="4"
                  :width="sampleRenderWidth"
                  height="8"
                  class="hover:fill-blue-300"
                  :fill="threadStateColor(sample.threadStateId)"
                  @click="clickSample(sample)"/>
          </svg>
        </div>
      </div>
    </div>

    <div class="w-full h-64" />
    <div class="fixed bottom-0 w-full bg-slate-300 h-64">
      <div v-for="frame in selectedFrames" :key="frame">{{ frame.typeName }}@{{ frame.methodName }}</div>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue } from 'vue-class-component'
import { Frame, Profile, Sample, StackTrace } from '@/models/jfr/profile'
import { readTextFile } from '@tauri-apps/api/fs'
import { invoke } from '@tauri-apps/api'

const CHART_WIDTH = 4096;
const CHART_HEIGHT = 2048;

export default class JFRView extends Vue {
  threads: {name: string, id: number}[] = []
  stackTracePool: {[id: number]: StackTrace} = {}
  threadStatePool: {[id: number]: string} = {}
  perThreadSamples: Map<number, Sample[]> = new Map<number, Sample[]>()
  profileDuration: {startMillis: number, endMillis: number} = {startMillis: 0, endMillis: 0}

  selectedFrames: Frame[] = []
  sampleRenderWidth = 0

  async mounted() {
    const filePath: string = await invoke("jfr_file_path")
    const text = await readTextFile(filePath)
    const profile: Profile = JSON.parse(text)
    const perThreadSamples = new Map<number, Sample[]>()
    this.stackTracePool = profile.stackTracePool
    this.threadStatePool = profile.threadStatePool

    let startMillis = Infinity
    let endMillis = -1

    profile.samples.forEach(sample => {
      startMillis = Math.min(startMillis, sample.timestamp)
      endMillis = Math.max(endMillis, sample.timestamp)

      if (perThreadSamples.has(sample.threadId)) {
        const samples = perThreadSamples.get(sample.threadId)!
        samples.push(sample)
      } else {
        perThreadSamples.set(sample.threadId, [sample])
      }
    })

    this.perThreadSamples = perThreadSamples
    this.profileDuration = {startMillis, endMillis}

    const threads: {name: string, id: number}[] = []
    let maxSamples = 0
    perThreadSamples.forEach((v, k) => {
      threads.push({name: profile.threadNamePool[k], id: k})
      maxSamples = Math.max(maxSamples, v.length)
    })

    this.threads = threads.sort((a, b) => a.name.localeCompare(b.name))

    this.sampleRenderWidth = CHART_WIDTH / maxSamples
  }

  threadStateColor(state: number): string {
    const stateName = this.threadStatePool[state]
    if (stateName === "STATE_RUNNABLE") {
      return "#6cba1e";
    }
    if (stateName === "STATE_SLEEPING") {
      return "#8d3eee";
    }
    return "#6f6d72";
  }

  chartArea(): {w: number, h: number} {
    return {w: CHART_WIDTH, h: CHART_HEIGHT}
  }

  clickSample(sample: Sample) {
    this.selectedFrames = this.stackTracePool[sample.stackTraceId].frames
  }
}
</script>
