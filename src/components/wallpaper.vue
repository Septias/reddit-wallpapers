<template lang="pug">
div.cursor-pointer(@click="select_wallpaper")
    img(:src="post.url")
</template>

<script lang="ts" setup>import { invoke } from '@tauri-apps/api/tauri'

const props = defineProps({
  post: {
    type: Object,
    required: true,
  },
})

function select_wallpaper() {
  invoke('select_wallpaper', { name: props.post.name })
}
</script>
