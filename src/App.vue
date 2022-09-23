<template>
  <div>
    <div class="h-12 fixed top-0 left-0 z-50 w-full bg-white border-b border-slate-400">
      <nav class="h-8 m-2 flex space-x-2">
        <router-link to="/execution-sample" class="hover:bg-slate-300 flex-none pl-2 pr-2 h-7 text-sm text-center border-2 rounded border-slate-400">
          <span class="align-middle">execution sample</span>
        </router-link>
        <router-link to="/jbm" class="hover:bg-slate-300 flex-none pl-2 pr-2 h-7 text-sm text-center border-2 rounded border-slate-400">
          <span class="align-middle">offcpu (jvm-blocking-monitor)</span>
        </router-link>
        <a href="https://github.com/ocadaruma/jfrv"
           target="_blank"
           class="absolute h-7 border-2 border-slate-500 pl-2 pr-2 right-2">&#x1f4d6; docs</a>
      </nav>
    </div>
    <div class="fixed top-12 left-0 right-0 bottom-0">
      <router-view />
    </div>
  </div>
</template>
