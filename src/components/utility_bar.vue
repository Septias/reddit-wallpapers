<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/tauri'

const props = defineProps({
  name: {
    type: String,
    required: true,
  },
})

function select_wallpaper() {
  invoke('select_wallpaper', { name: props.name })
}
</script>

<template>
  <div>
    <div i-carbon:select-01 class="cursor-pointer text-lg" @click="select_wallpaper" />
  </div>
</template>
