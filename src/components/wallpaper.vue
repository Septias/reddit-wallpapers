<script lang="ts" setup async>
import { join } from '@tauri-apps/api/path'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
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

const emit = defineEmits(['removed'])
let thumbnail_path
  = await convertFileSrc(await join(props.basePath, 'thumbnails', props.post.file_name))

watch(props.post, async () => {
  thumbnail_path = await convertFileSrc(await join(props.basePath, 'thumbnails', props.post.file_name))
})

function select_wallpaper() {
  invoke('select_wallpaper', { name: props.post.name })
}

async function remove_wallpaper(event: Event) {
  event.stopPropagation()
  try {
    await invoke('remove_wallpaper', { name: props.post.name })
    emit('removed', props.post.name)
  } catch (error) {
    console.error('Failed to remove wallpaper:', error)
  }
}

async function remove_wallpaper_local(event: Event) {
  event.stopPropagation()
  try {
    await invoke('remove_wallpaper_local', { name: props.post.name })
    emit('removed', props.post.name)
  } catch (error) {
    console.error('Failed to remove wallpaper locally:', error)
  }
}
</script>

<template lang="pug">
.relative.cursor-pointer.group(@click="select_wallpaper")
  img(:src="thumbnail_path")
  .absolute.top-1.right-1.opacity-0.transition-opacity.flex.gap-1(class="group-hover:opacity-100")
    button.bg-orange-500.text-white.rounded.p-1.transition-colors(
      @click="remove_wallpaper_local"
      title="Remove wallpaper locally"
      class="hover:bg-orange-600"
    )
      .i-carbon-folder-off.w-4.h-4
    button.bg-red-500.text-white.rounded.p-1.transition-colors(
      @click="remove_wallpaper"
      title="Remove wallpaper"
      class="hover:bg-red-600"
    )
      .i-carbon-trash-can.w-4.h-4
</template>
