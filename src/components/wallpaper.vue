<script lang="ts" setup async>
import { join } from '@tauri-apps/api/path'
import { convertFileSrc, invoke } from '@tauri-apps/api/tauri'
const props = defineProps({
  post: {
    type: Object,
    required: true,
  },
  basePath: {
    type: String,
    required: true,
  },
})
let thumbnail_path
  = await convertFileSrc(await join(props.basePath, 'thumbnails', props.post.file_name))

watch(props.post, async () => {
  thumbnail_path = await convertFileSrc(await join(props.basePath, 'thumbnails', props.post.file_name))
})

function select_wallpaper() {
  invoke('select_wallpaper', { name: props.post.name })
}
</script>

<template lang="pug">
div.cursor-pointer(@click="select_wallpaper")
  img(:src="thumbnail_path")
</template>
