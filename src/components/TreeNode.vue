<template>
  <div>
    <div v-for="(node, index) in data" :key="index">
      <span @click="toggle(index)" :class="'whitespace-pre' + (node.columns ? ' cursor-pointer' : '')">
        {{ node.columns ? (isOpen[index] ? '[-]' : '[+]') : '      *' }} {{ node.type ? `${node.name}: ${node.type}` : node.name }}
      </span>
      <TreeNode
          v-if="isOpen[index] && node.columns"
          :data="node.columns"
          class="whitespace-nowrap"
      />
    </div>
  </div>
</template>

<script setup>
import { ref, defineProps } from 'vue'

const props = defineProps({
  data: Array,
});

const isOpen = ref({});

function toggle(index) {
  isOpen.value[index] = !isOpen.value[index];
}
</script>
