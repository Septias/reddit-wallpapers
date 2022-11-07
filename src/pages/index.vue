<script setup lang="ts" async>
// import { posts } from '~/logic/post_mock'
import { invoke } from '@tauri-apps/api/tauri'
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

onMounted(() => {
  update()
})
</script>

<template lang="pug">
router-link.absolute.top-0.left-0.bg-primaryl.p-1.rounded.m-1(to="/config")
  div.text-white.i-carbon-settings
div.p-2.wallpapers.grid.gap-2.justify-center.items-center
  div(v-for="post in posts" :key="post.name")
    wallpaper(:post="post" :basePath="base_path")
</template>

<style lang="sass">
#divider
  border-right: 1px solid #ccc

.wallpapers
  grid-template-columns: repeat(auto-fit, 300px)
</style>
