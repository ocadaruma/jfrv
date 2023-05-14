import {FlameGraph, FlameGraphConfig} from "../../jfrv-wasm/pkg";
import router from "@/router";

const OBJECT_ID = "__flame_graph__"

export const FLAME_GRAPH_CONFIG: FlameGraphConfig = {
  chartId: "flame-graph",
  highlightId: "highlight",
  highlightTextId: "highlight-text",
  statusId: "status",
  font: "12px Verdana, sans-serif",
  colorPalette: {
    "Interpreted": { baseHex: 0xb2e1b2, rMix: 20, gMix: 20, bMix: 20 },
    "JitCompiled": { baseHex: 0x50e150, rMix: 30, gMix: 30, bMix: 30 },
    "Inlined": { baseHex: 0x50cccc, rMix: 30, gMix: 30, bMix: 30 },
    "Native": { baseHex: 0xe15a5a, rMix: 30, gMix: 40, bMix: 40 },
    "Cpp": { baseHex: 0xc8c83c, rMix: 30, gMix: 30, bMix: 10 },
    "Kernel": { baseHex: 0xe17d00, rMix: 30, gMix: 30, bMix: 0 },
    "C1Compiled": { baseHex: 0xcce880, rMix: 20, gMix: 20, bMix: 20 },
    "Unknown": { baseHex: 0, rMix: 0, gMix: 0, bMix: 0 },
  }
}

export class FlameGraphWindow {
  delegate: Window
  objectElement: HTMLObjectElement

  constructor(delegate: Window, objectElement: HTMLObjectElement) {
    this.delegate = delegate
    this.objectElement = objectElement
  }

  static open(flameGraph: FlameGraph) {
    const url = `${process.env.BASE_URL}${router.resolve({name: "flame-graph"}).href}`
    const w = window.open(url, "_blank")
    if (w) {
      w.onload = () => {
        const obj = w.document.createElement("object")
        obj.type = "application/json"
        obj.data = `data:application/json;base64,${btoa(JSON.stringify(flameGraph))}`
        obj.id = OBJECT_ID
        obj.style.display = "none"
        w.document.body.appendChild(obj)
      }
    }
  }

  static async flameGraph(): Promise<FlameGraph> {
    const obj = window.document.getElementById(OBJECT_ID) as HTMLObjectElement
    const res = await fetch(obj.data)
    const json = await res.json()
    return json as FlameGraph
  }
}
