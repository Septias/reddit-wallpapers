<script setup lang="ts" async>
// import { posts } from '~/logic/post_mock'
import { invoke } from '@tauri-apps/api/core'
import NProgress from 'nprogress'

const posts = ref(await invoke('get_cached_wallpapers') as Post[])
const base_path: string = await invoke('get_wallpapers_path')

interface Post {
  name: string
  title: string
  url: string
}

async function update() {
  NProgress.start()
  await invoke('fetch_recent')
  NProgress.done()
  posts.value = await invoke('get_cached_wallpapers')
}

function handleWallpaperRemoved(removedName: string) {
  posts.value = posts.value.filter(post => post.name !== removedName)
}

onMounted(() => {
  update()
})
</script>

<template lang="pug">
.flex.items-center.justify-between.p-2.bg-primaryl
  router-link.bg-primary.p-2.rounded(to="/config")
    div.text-white.i-carbon-settings
  div.text-white.font-bold.text-lg Reddit Wallpapers
  div.w-8
div.p-2.wallpapers.grid.gap-2.justify-center.items-center
  div(v-for="post in posts" :key="post.name")
    wallpaper(:post="post" :basePath="base_path" @removed="handleWallpaperRemoved")
</template>

<style lang="sass">
#divider
  border-right: 1px solid #ccc

.wallpapers
  grid-template-columns: repeat(auto-fit, 300px)
</style>
