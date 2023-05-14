<template>
  <div class="m-0 p-[10px] bg-white" :style="`font: ${FLAME_GRAPH_CONFIG.font}`">
    <canvas id="flame-graph"></canvas>
    <div id="highlight">
      <span id="highlight-text"></span>
    </div>
    <p class="mt-[5px] mr-0 mb-[5px] ml-0" id="status"></p>
  </div>
</template>

<script lang="ts" setup>
import {onMounted} from "vue";
import {FLAME_GRAPH_CONFIG, FlameGraphWindow} from "@/views/flame-graph";

onMounted(async () => {
  window.addEventListener("load", async () => {
    const wasm = await import("../../jfrv-wasm/pkg")
    const flameGraph = await FlameGraphWindow.flameGraph()
    const r = new wasm.FlameGraphRenderer(flameGraph, FLAME_GRAPH_CONFIG)
    window.onmousemove = (e) => {
      r.onmousemove(e)
    }
    window.onmouseout = (e) => {
      r.onmouseout(e)
    }
    window.onclick = (e) => {
      r.onclick(e)
    }
    r.render()
  })
})
</script>

<style scoped lang="css">
#highlight {
  position: absolute;
  display: none;
  overflow: hidden;
  white-space: nowrap;
  pointer-events: none;
  background-color: #ffffe0;
  outline: 1px solid #ffc000;
  height: 15px;
}

#highlight-text {
  padding: 0 3px 0 3px;
}

#status {
  overflow: hidden;
  white-space: nowrap;
  position: sticky;
  background: #ffffff;
  bottom: 0;
}

#flame-graph {
  width: 100%;
}
</style>
