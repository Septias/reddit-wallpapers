<script setup lang="ts" async>

import { invoke } from '@tauri-apps/api/tauri'
let posts: Post[] = await invoke('get_cached_wallpapers')

// import { posts } from '~/logic/post_mock'

interface Post {
  name: string
  title: string
  url: string
}

async function update() {
  await invoke('fetch_recent')
  posts = await invoke('get_cached_wallpapers')
}

</script>

<template lang="pug">
div(class="grid m-4 grid-cols-3 gap-4")
  div(v-for="post in posts" class="border pl-2 rounded flex")
    div(class="pb-2 pr-1")
      h2(class="p-2") {{ post.title }}
      img(:src="post.url")
    div#divider
    utility_bar(:name="post.name")

div(class="absolute top-0 left-0")
  div(class="p-1 border rounded cursor-pointer")
    div(@click="update") +
</template>

<style lang="sass">
#divider
  border-right: 1px solid #ccc
</style>
