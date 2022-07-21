<script setup lang="ts" async>

// import { posts } from '~/logic/post_mock'
import { invoke } from '@tauri-apps/api/tauri'
const posts = ref(await invoke('get_cached_wallpapers') as Post[])
const fetching = ref(false)
const base_path: string = await invoke('get_wallpapers_path')

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
  // update()
})

</script>

<template lang="pug">
div.absolute.right-0.m-2.p-2.rounded.text-pink-500.cursor-pointer(@click="update")
  div(class="i-carbon:cloud-download" :class="{ 'rotate': fetching }")

div.p-2.wallpapers.grid.gap-2.justify-center
  div(v-for="post in posts" :key="post.name")
    wallpaper(:post="post" :basePath="base_path")
</template>

<style lang="sass">
#divider
  border-right: 1px solid #ccc

.wallpapers
  grid-template-columns: repeat(auto-fit, 300px)

.rotate
  animation: rotation 0.25s infinite linear

@keyframes rotation
  from
    transform: rotate(0deg)
  to
    transform: rotate(359deg)

</style>
