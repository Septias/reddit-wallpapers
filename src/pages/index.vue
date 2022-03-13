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
  console.log('hi')

  await invoke('fetch_recent')
  console.log('hi zwei')

  posts = await invoke('get_cached_wallpapers')
}

</script>

<template lang="pug">
div.p-4.flex.justify-between.bg-blue.items-center.text-white
  h1.text-5xl Wallpapers
  div.p-1.border.rounded.cursor-pointer(@click="update")
    div(class="i-carbon:cloud-download")

div.grid.m-4.grid-cols-3.gap-4
  div.border.pl-2.rounded.flex(v-for="post in posts")
    div.pb-2.pr-1
      h2.p-2 {{ post.title }}
      img(:src="post.url")
    div#divider
    utility_bar(:name="post.name")

</template>

<style lang="sass">
#divider
  border-right: 1px solid #ccc
</style>
