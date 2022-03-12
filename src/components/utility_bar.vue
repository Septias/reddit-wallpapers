<template>
  <div>
    <div i-carbon-add class="cursor-pointer" @click="select_wallpaper" />
    <div i-carbon-language />
  </div>
</template>

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
<!-- div(class="p-1")
  form
    div(i = "carbon-sun")
    input(type="checkbox") -->
