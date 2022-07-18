<template>
  <div>
    <div class="h-6"></div>
    <nav class="h-6 fixed top-0 left-0 z-50 bg-slate-200 w-full">
      <router-link to="/">Home</router-link> |
      <router-link to="/jfr">JFR Viewer</router-link>
    </nav>
    <router-view/>
  </div>
</template>
