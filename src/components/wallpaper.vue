<template lang="pug">
div.cursor-pointer(@click="select_wallpaper")
    img(:src="converted")
</template>

<script lang="ts" setup async>
import { convertFileSrc, invoke } from '@tauri-apps/api/tauri'
const props = defineProps({
  post: {
    type: Object,
    required: true,
  },
})

const converted = await convertFileSrc(props.post.path)

function select_wallpaper() {
  invoke('select_wallpaper', { name: props.post.name })
}
</script>
