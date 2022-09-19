const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const path = require("path");

const { defineConfig } = require('@vue/cli-service')
module.exports = defineConfig({
  transpileDependencies: true,
  chainWebpack: config => {
    config.plugin("wasm-pack")
      .use(WasmPackPlugin)
      .init(Plugin =>
        new Plugin({
          crateDirectory: path.resolve(__dirname, "jfrv-wasm"),
          // https://github.com/wasm-tool/wasm-pack-plugin/issues/93
          outDir: path.resolve(__dirname, "jfrv-wasm/pkg"),
          // temporary
          forceMode: "production"
        })
      ).end()
  },
  configureWebpack: {
    experiments: {
      asyncWebAssembly: true
    }
  },
  publicPath: process.env.NODE_ENV === "production" ? "/jfrv/" : "/"
})
