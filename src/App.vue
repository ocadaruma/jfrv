<template>
  <div>
    <div class="h-12 fixed top-0 left-0 z-50 w-full bg-white border-b border-slate-400">
      <nav class="h-8 m-2 flex space-x-2">
        <router-link to="/home" class="hover:bg-slate-300 flex-none w-24 h-7 text-sm text-center border-2 rounded border-slate-400">
          <span class="align-middle">home</span>
        </router-link>
        <router-link to="/jfr" class="hover:bg-slate-300 flex-none w-24 h-7 text-sm text-center border-2 rounded border-slate-400">
          <span class="align-middle">jfr</span>
        </router-link>
      </nav>
    </div>
    <div class="fixed top-12 left-0 right-0 bottom-0">
      <router-view />
    </div>
  </div>
</template>
