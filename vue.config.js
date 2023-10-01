const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const path = require("path");

const { defineConfig } = require('@vue/cli-service')
module.exports = defineConfig({
  // transpileDependencies: false,
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
    config.module
      .rule("vue")
      .use("vue-loader")
      .tap(options => ({
        ...options,
        compilerOptions: {
          isCustomElement: tag => tag.startsWith("perspective-"),
        }
      }))
  },
  configureWebpack: {
    experiments: {
      asyncWebAssembly: true,
    },
    module: {
      rules: [
        {
          test: /.*duckdb-mvp\.wasm$/,
          type: 'asset/resource',
          generator: {
              filename: 'static/wasm/[name].[contenthash][ext]',
          },
        },
      ]
    },
  },
  publicPath: process.env.NODE_ENV === "production" ? "/jfrv/" : "/"
})
