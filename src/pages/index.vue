<script setup lang="ts" async>

import { invoke } from '@tauri-apps/api/tauri'
const posts = ref(await invoke('get_cached_wallpapers') as Post[])
const fetching = ref(false)
// import { posts } from '~/logic/post_mock'

interface Post {
  name: string
  title: string
  url: string
}

async function update() {
  fetching.value = true
  await invoke('fetch_recent')
  posts.value = await invoke('get_cached_wallpapers')
  fetching.value = false
}

onMounted(() => {
  update()
})

</script>

<template lang="pug">
div.p-4.flex.justify-between.bg-blue-900.items-center.text-white
  h1.text-5xl Wallpapers
  div.p-1.border.rounded.cursor-pointer(@click="update")
    div(class="i-carbon:cloud-download" :class="{ 'rotate': fetching }")

div.p-2.wallpapers.grid.gap-2
  div.border.rounded.flex.max-width(v-for="post in posts")
    wallpaper(:post="post")
</template>

<style lang="sass">
#divider
  border-right: 1px solid #ccc

.wallpapers
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr))

.rotate
  animation: rotation 0.25s infinite linear

@keyframes rotation
  from
    transform: rotate(0deg)
  to
    transform: rotate(359deg)

</style>
