/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    // We need to resolve to absolute path here.
    // We found that changed-file detection fails if we use relative path as following:
    //
    // 1. Kick vue-cli-service tauri:serve
    // 2. tauri:serve starts dev-server and invokes tauri-cli node-binding's `run`
    //   * dev-server and tauri-cli runs in same process without forking. This is the point.
    //   * tauri-cli changes the working directory to src-tauri (https://github.com/tauri-apps/tauri/blob/v1.0.4/tooling/cli/src/dev.rs#L85)
    // 3. When we modify some vue source files, dev-server's hot reload is kicked (by inotify? don't know the details),
    //    then tailwindcss/setupTrackingContext.js is called eventually, and it fails by ENOENT because the chdir is src-tauri at this point.
    require("path").resolve(__dirname, "./src/**/*.{vue,js,ts,jsx,tsx}"),
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
