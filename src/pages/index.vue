<script setup lang="ts" async>
// import { posts } from '~/logic/post_mock'
import { invoke } from '@tauri-apps/api/tauri'

if (!await invoke('is_configured')) {
  const router = useRouter()
  router.push('/config')
}

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
  update()
})
</script>

<template lang="pug">
router-link.absolute.top-0.left-0.bg-primaryl.p-2.rounded.m-1(to="/config")
  span.i-mdi-cog.text-white
div.p-2.wallpapers.grid.gap-2.justify-center
  div(v-for="post in posts" :key="post.name")
    wallpaper(:post="post" :basePath="base_path")
</template>

<style lang="sass">
#divider
  border-right: 1px solid #ccc

.wallpapers
  grid-template-columns: repeat(auto-fit, 300px)
</style>
